pragma solidity ^0.6.6;

contract OodsPoly \{
    fallback() external \{
        assembly \{
            let res := 0
            let PRIME := 0x800000000000011000000000000000000000000000000000000000000000001
            // NOTE - If compilation hits a stack depth error on variable PRIME,
            // then uncomment the following line and globally replace PRIME
            // with  `mload({modulus_location})`
            // mstore({modulus_location}, 0x800000000000011000000000000000000000000000000000000000000000001)
            // Copy input from calldata to memory.
            calldatacopy(0x0, 0x0, {input_data_size})

            function expmod(base, exponent, modulus) -> result \{
                let p := {expmod_context}
                mstore(p, 0x20) // Length of Base
                mstore(add(p, 0x20), 0x20) // Length of Exponent
                mstore(add(p, 0x40), 0x20) // Length of Modulus
                mstore(add(p, 0x60), base) // Base
                mstore(add(p, 0x80), exponent) // Exponent
                mstore(add(p, 0xa0), modulus) // Modulus
                // call modexp precompile
                if iszero(call(not(0), 0x05, 0, p, 0xc0, p, 0x20)) \{
                    revert(0, 0)
                }
                result := mload(p)
            }

            function degree_adjustment(
                composition_polynomial_degree_bound,
                constraint_degree,
                numerator_degree,
                denominator_degree
            ) -> result \{
                result := sub(
                    sub(composition_polynomial_degree_bound, 1),
                    sub(
                        add(constraint_degree, numerator_degree),
                        denominator_degree
                    )
                )
            }

            function small_expmod(x, num, prime) -> result \{
                result := 1
                for \{ let ind := 0 } lt(ind, num) \{ ind := add(ind, 1) } \{
                    result := mulmod(result, x, prime)
                }
            }

            // Store adjustment degrees
            {{ for da in degree_adjustments -}}
            mstore({da.location}, expmod({x}, {da.exponent}, {modulus}))
            {{ endfor }}

            // Store the values which will be batch inverted
            {{ for bi in batch_inverted -}}
            mstore({bi.location}, {bi.expression})
            {{ endfor }}

            // Compute batch inversion
            \{
                // Compute the inverses of the denominators into denominator_invs using batch inverse.

                // Start by computing the cumulative product.
                // Let (d_0, d_1, d_2, ..., d_\{n-1}) be the values in denominators. Then after this loop
                // denominator_invs will be (1, d_0, d_0 * d_1, ...) and prod will contain the value of
                // d_0 * ... * d_\{n-1}.
                // Compute the offset between the partial_products array and the input values array.
                let products_to_values := {products_to_values}
                let prod := 1
                let partial_product_end_ptr := {partial_product_end_ptr}
                for \{ let partial_product_ptr := {partial_product_start_ptr} }
                    lt(partial_product_ptr, partial_product_end_ptr)
                    \{ partial_product_ptr := add(partial_product_ptr, 0x20) } \{
                    mstore(partial_product_ptr, prod)
                    // prod *= d_i.
                    prod := mulmod(prod, mload(add(partial_product_ptr, products_to_values)), {modulus})
                }

                let first_partial_product_ptr := {first_partial_product_ptr}
                // Compute the inverse of the product.
                let prod_inv := expmod(prod, sub({modulus}, 2), {modulus})

                // Compute the inverses.
                // Loop over denominator_invs in reverse order.
                // current_partial_product_ptr is initialized to one past the end.
                for \{
                    let current_partial_product_ptr := {last_partial_product_ptr}
                } gt(current_partial_product_ptr, first_partial_product_ptr) \{

                } \{
                    current_partial_product_ptr := sub(
                        current_partial_product_ptr,
                        0x20
                    )
                    // Store 1/d_i = (d_0 * ... * d_\{i-1}) * 1/(d_0 * ... * d_i).
                    mstore(
                        current_partial_product_ptr,
                        mulmod(
                            mload(current_partial_product_ptr),
                            prod_inv,
                            {modulus}
                        )
                    )
                    // Update prod_inv to be 1/(d_0 * ... * d_\{i-1}) by multiplying by d_i.
                    prod_inv := mulmod(
                        prod_inv,
                        mload(
                            add(current_partial_product_ptr, products_to_values)
                        ),
                        {modulus}
                    )
                }
            }

            // Sum constraint polynomials
            {{ for c in constraints -}}
            \{
                let val := {c.expression}
                res := addmod(res, mulmod(val, add(mload({c.first_coefficient_location}), mulmod(mload({c.second_coefficient_location}), {c.degree_adjustment_location}, {modulus})), {modulus}), {modulus})
            }
            {{ endfor -}}

            // Return result
            mstore(0, res)
            return(0, 0x20)
        }
    }
}