mod binary;
use crate::Zero;

pub trait AddInline<Rhs = Self>: Sized {
    /// **Note.** Implementers *must* add the `#[inline(always)]` attribute
    fn add_inline(&self, rhs: Rhs) -> Self;

    /// **Note.** Implementers *must* add the `#[inline(always)]` attribute
    /// By default it redirects to the non-assigned version
    #[inline(always)]
    fn add_assign_inline(&mut self, rhs: Rhs) {
        *self = self.add_inline(rhs)
    }

    // Optionally-inline version
    #[cfg_attr(feature = "inline", inline(always))]
    fn add(&self, rhs: Rhs) -> Self {
        self.add_inline(rhs)
    }

    // Optionally-inline version
    #[cfg_attr(feature = "inline", inline(always))]
    fn add_assign(&mut self, rhs: Rhs) {
        self.add_assign_inline(rhs)
    }
}

pub trait AddFullInline<Rhs = Self>: Sized {
    type High;

    /// **Note.** Implementers *must* add the `#[inline(always)]` attribute
    fn add_full_inline(&self, rhs: Rhs) -> (Self, Self::High);

    /// **Note.** Implementers *must* add the `#[inline(always)]` attribute
    /// By default it redirects to the non-assigned version
    #[inline(always)]
    fn add_full_assign_inline(&mut self, rhs: Rhs) -> Self::High {
        let (lo, hi) = self.add_full_inline(rhs);
        *self = lo;
        hi
    }

    // Optionally-inline version
    #[cfg_attr(feature = "inline", inline(always))]
    fn add_full(&self, rhs: Rhs) -> (Self, Self::High) {
        self.add_full_inline(rhs)
    }

    // Optionally-inline version
    #[cfg_attr(feature = "inline", inline(always))]
    fn add_full_assign(&mut self, rhs: Rhs) -> Self::High {
        self.add_full_assign_inline(rhs)
    }
}

pub trait SquareInline: Sized {
    /// **Note.** Implementers *must* add the `#[inline(always)]` attribute
    // Default implementation to be overridden
    fn square_inline(&self) -> Self;

    #[inline(always)]
    fn square_assign_inline(&mut self) {
        *self = self.square_inline()
    }

    // Optionally-inline version
    #[cfg_attr(feature = "inline", inline(always))]
    fn square(&self) -> Self {
        self.square_inline()
    }

    #[cfg_attr(feature = "inline", inline(always))]
    fn square_assign(&mut self) {
        self.square_assign_inline()
    }
}

pub trait SquareFullInline: Sized {
    /// **Note.** Implementers *must* add the `#[inline(always)]` attribute
    fn square_full_inline(&self) -> (Self, Self);

    #[inline(always)]
    fn square_full_assign_inline(&mut self) -> Self {
        let (lo, hi) = self.square_full_inline();
        *self = lo;
        hi
    }

    // Optionally-inline version
    #[cfg_attr(feature = "inline", inline(always))]
    fn square_full(&self) -> (Self, Self) {
        self.square_full_inline()
    }

    // Optionally-inline version
    #[cfg_attr(feature = "inline", inline(always))]
    fn square_full_assign(&mut self) -> Self {
        self.square_full_assign_inline()
    }
}

pub trait MulInline<Rhs>: Sized {
    /// **Note.** Implementers *must* add the `#[inline(always)]` attribute
    fn mul_inline(&self, rhs: Rhs) -> Self;

    /// **Note.** Implementers *must* add the `#[inline(always)]` attribute
    /// By default it redirects to the non-assigned version
    #[inline(always)]
    fn mul_assign_inline(&mut self, rhs: Rhs) {
        *self = self.mul_inline(rhs)
    }

    // Optionally-inline version
    #[cfg_attr(feature = "inline", inline(always))]
    fn mul(&self, rhs: Rhs) -> Self {
        self.mul_inline(rhs)
    }

    // Optionally-inline version
    #[cfg_attr(feature = "inline", inline(always))]
    fn mul_assign(&mut self, rhs: Rhs) {
        self.mul_assign_inline(rhs)
    }
}

pub trait MulFullInline<Rhs>: Sized {
    type High;

    /// **Note.** Implementers *must* add the `#[inline(always)]` attribute
    fn mul_full_inline(&self, rhs: Rhs) -> (Self, Self::High);


    /// **Note.** Implementers *must* add the `#[inline(always)]` attribute
    #[inline(always)]
    fn mul_full_assign_inline(&mut self, rhs: Rhs) -> Self::High {
        let (lo, hi) = self.mul_full_inline(rhs);
        *self = lo;
        hi
    }

    // Optionally-inline version
    #[cfg_attr(feature = "inline", inline(always))]
    fn mul_full(&self, rhs: Rhs) -> (Self, Self::High) {
        self.mul_full_inline(rhs)
    }

    // Optionally-inline version
    #[cfg_attr(feature = "inline", inline(always))]
    fn mul_full_assign(&self, rhs: Rhs) -> (Self, Self::High) {
        self.mul_full_inline(rhs)
    }
}

pub trait DivRem<Rhs> {
    type Quotient;
    type Remainder;

    fn div_rem(&self, rhs: Rhs) -> Option<(Self::Quotient, Self::Remainder)>;
}

pub trait InvMod: Sized {
    fn inv_mod(&self, modulus: &Self) -> Option<Self>;
}

pub trait GCD: Sized {
    fn gcd(a: &Self, b: &Self) -> Self;

    fn gcd_extended(a: &Self, b: &Self) -> (Self, Self, Self, bool);

    // TODO: LCM
}

// TODO: Automatically derive Mul<..> traits. Maybe also MulAssign<..>

// `T` can not have interior mutability.
#[allow(clippy::declare_interior_mutable_const)]
pub trait MontgomeryParameters<T: Sized> {
    /// The modulus to implement in Montgomery form
    const MODULUS: T;

    /// M64 = -MODULUS^(-1) mod 2^64
    const M64: u64;

    // R1 = 2^256 mod MODULUS
    const R1: T;

    // R2 = 2^512 mod MODULUS
    const R2: T;

    // R3 = 2^768 mod MODULUS
    const R3: T;
}

pub trait Montgomery: Zero {
    fn to_montgomery<M: MontgomeryParameters<Self>>(&self) -> Self {
        // `Self` should not have interior mutability.
        #[allow(clippy::borrow_interior_mutable_const)]
        self.mul_redc::<M>(&M::R2)
    }

    fn from_montgomery<M: MontgomeryParameters<Self>>(&self) -> Self {
        // Use inline version to propagate the zeros
        Self::redc_inline::<M>(self, &Self::zero())
    }

    /// **Note.** Implementers *must* add the `#[inline(always)]` attribute
    fn redc_inline<M: MontgomeryParameters<Self>>(lo: &Self, hi: &Self) -> Self;

    /// **Note.** Implementers *must* add the `#[inline(always)]` attribute
    fn square_redc_inline<M: MontgomeryParameters<Self>>(&self) -> Self;

    /// **Note.** Implementers *must* add the `#[inline(always)]` attribute
    fn mul_redc_inline<M: MontgomeryParameters<Self>>(&self, rhs: &Self) -> Self;

    fn inv_redc<M: MontgomeryParameters<Self>>(&self) -> Option<Self>;

    fn square_redc<M: MontgomeryParameters<Self>>(&self) -> Self {
        self.square_redc_inline::<M>()
    }

    fn mul_redc<M: MontgomeryParameters<Self>>(&self, rhs: &Self) -> Self {
        self.mul_redc_inline::<M>(rhs)
    }

    /// Multiply two numbers in non-Montgomery form.
    ///
    /// Combined `to_montgomery`, `mul_redc`, and `from_montgomery`.
    ///
    /// Normally this would require four `mul_redc` operations, but two
    /// of them cancel out, making this an efficient way to do a single
    /// modular multiplication.
    ///
    /// # Requirements
    /// Inputs are required to be reduced modulo `M::MODULUS`.
    fn mul_mod<M: MontgomeryParameters<Self>>(&self, rhs: &Self) -> Self {
        // OPT: Is this better than Barret reduction?
        // We want to borrow `&M::R2` as const. `Self` should not have interior
        // mutability.
        #[allow(clippy::borrow_interior_mutable_const)]
        let mont = Self::mul_redc_inline::<M>(self, &M::R2);
        Self::mul_redc_inline::<M>(&mont, rhs)
    }
}

// TODO: Mega-trait for binary rings like U256 that PrimeField can use

// False positive, we re-export the trait.
#[allow(unreachable_pub)]
pub use binary::{Binary, BinaryAssignRef, BinaryOps};

pub trait BinaryRing: Binary {}

// TODO: Factorial, Totient, Carmichael, Jacobi, Legendre, Binomial, etc.
// See https://gmplib.org/manual/Number-Theoretic-Functions.html
