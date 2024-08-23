//! Shared ISLE prelude implementation for optimization (mid-end) and
//! lowering (backend) ISLE environments.

/// Helper macro to define methods in `prelude.isle` within `impl Context for
/// ...` for each backend. These methods are shared amongst all backends.
#[macro_export]
#[doc(hidden)]
macro_rules! isle_common_prelude_methods {
    () => {
        /// We don't have a way of making a `()` value in isle directly.
        #[inline]
        fn unit(&mut self) -> Unit {
            ()
        }

        #[inline]
        fn u8_as_u32(&mut self, x: u8) -> u32 {
            x.into()
        }

        #[inline]
        fn u8_as_u64(&mut self, x: u8) -> u64 {
            x.into()
        }

        #[inline]
        fn u16_as_u64(&mut self, x: u16) -> u64 {
            x.into()
        }

        #[inline]
        fn u32_as_u64(&mut self, x: u32) -> u64 {
            x.into()
        }

        #[inline]
        fn i64_as_u64(&mut self, x: i64) -> u64 {
            x as u64
        }

        #[inline]
        fn u64_as_i32(&mut self, x: u64) -> i32 {
            x as i32
        }

        #[inline]
        fn i64_neg(&mut self, x: i64) -> i64 {
            x.wrapping_neg()
        }

        #[inline]
        fn u64_add(&mut self, x: u64, y: u64) -> u64 {
            x.wrapping_add(y)
        }

        #[inline]
        fn u64_sub(&mut self, x: u64, y: u64) -> u64 {
            x.wrapping_sub(y)
        }

        #[inline]
        fn u64_mul(&mut self, x: u64, y: u64) -> u64 {
            x.wrapping_mul(y)
        }

        #[inline]
        fn u64_sdiv(&mut self, x: u64, y: u64) -> Option<u64> {
            let x = x as i64;
            let y = y as i64;
            x.checked_div(y).map(|d| d as u64)
        }

        #[inline]
        fn u64_udiv(&mut self, x: u64, y: u64) -> Option<u64> {
            x.checked_div(y)
        }

        #[inline]
        fn u64_and(&mut self, x: u64, y: u64) -> u64 {
            x & y
        }

        #[inline]
        fn u64_or(&mut self, x: u64, y: u64) -> u64 {
            x | y
        }

        #[inline]
        fn u64_xor(&mut self, x: u64, y: u64) -> u64 {
            x ^ y
        }

        #[inline]
        fn u64_shl(&mut self, x: u64, y: u64) -> u64 {
            x << y
        }

        #[inline]
        fn imm64_shl(&mut self, ty: Type, x: Imm64, y: Imm64) -> Imm64 {
            // Mask off any excess shift bits.
            let shift_mask = (ty.bits() - 1) as u64;
            let y = (y.bits() as u64) & shift_mask;

            // Mask the result to `ty` bits.
            let ty_mask = self.ty_mask(ty) as i64;
            Imm64::new((x.bits() << y) & ty_mask)
        }

        #[inline]
        fn imm64_ushr(&mut self, ty: Type, x: Imm64, y: Imm64) -> Imm64 {
            let ty_mask = self.ty_mask(ty);
            let x = (x.bits() as u64) & ty_mask;

            // Mask off any excess shift bits.
            let shift_mask = (ty.bits() - 1) as u64;
            let y = (y.bits() as u64) & shift_mask;

            // NB: No need to mask off high bits because they are already zero.
            Imm64::new((x >> y) as i64)
        }

        #[inline]
        fn imm64_sshr(&mut self, ty: Type, x: Imm64, y: Imm64) -> Imm64 {
            // Sign extend `x` from `ty.bits()`-width to the full 64 bits.
            let shift = u32::checked_sub(64, ty.bits()).unwrap_or(0);
            let x = (x.bits() << shift) >> shift;

            // Mask off any excess shift bits.
            let shift_mask = (ty.bits() - 1) as i64;
            let y = y.bits() & shift_mask;

            // Mask off sign bits that aren't part of `ty`.
            let ty_mask = self.ty_mask(ty) as i64;
            Imm64::new((x >> y) & ty_mask)
        }

        #[inline]
        fn u64_not(&mut self, x: u64) -> u64 {
            !x
        }

        #[inline]
        fn u64_eq(&mut self, x: u64, y: u64) -> bool {
            x == y
        }

        #[inline]
        fn u64_le(&mut self, x: u64, y: u64) -> bool {
            x <= y
        }

        #[inline]
        fn u64_lt(&mut self, x: u64, y: u64) -> bool {
            x < y
        }

        #[inline]
        fn u64_is_zero(&mut self, value: u64) -> bool {
            0 == value
        }

        #[inline]
        fn u64_is_odd(&mut self, x: u64) -> bool {
            x & 1 == 1
        }

        #[inline]
        fn i64_sextend_imm64(&mut self, ty: Type, mut x: Imm64) -> i64 {
            x.sign_extend_from_width(ty.bits());
            x.bits()
        }

        #[inline]
        fn u64_uextend_imm64(&mut self, ty: Type, x: Imm64) -> u64 {
            (x.bits() as u64) & self.ty_mask(ty)
        }

        #[inline]
        fn imm64_icmp(&mut self, ty: Type, cc: &IntCC, x: Imm64, y: Imm64) -> Imm64 {
            let ux = self.u64_uextend_imm64(ty, x);
            let uy = self.u64_uextend_imm64(ty, y);
            let sx = self.i64_sextend_imm64(ty, x);
            let sy = self.i64_sextend_imm64(ty, y);
            let result = match cc {
                IntCC::Equal => ux == uy,
                IntCC::NotEqual => ux != uy,
                IntCC::UnsignedGreaterThanOrEqual => ux >= uy,
                IntCC::UnsignedGreaterThan => ux > uy,
                IntCC::UnsignedLessThanOrEqual => ux <= uy,
                IntCC::UnsignedLessThan => ux < uy,
                IntCC::SignedGreaterThanOrEqual => sx >= sy,
                IntCC::SignedGreaterThan => sx > sy,
                IntCC::SignedLessThanOrEqual => sx <= sy,
                IntCC::SignedLessThan => sx < sy,
            };
            Imm64::new(result.into())
        }

        #[inline]
        fn ty_bits(&mut self, ty: Type) -> u8 {
            use std::convert::TryInto;
            ty.bits().try_into().unwrap()
        }

        #[inline]
        fn ty_bits_u16(&mut self, ty: Type) -> u16 {
            ty.bits() as u16
        }

        #[inline]
        fn ty_bits_u64(&mut self, ty: Type) -> u64 {
            ty.bits() as u64
        }

        #[inline]
        fn ty_bytes(&mut self, ty: Type) -> u16 {
            u16::try_from(ty.bytes()).unwrap()
        }

        #[inline]
        fn ty_mask(&mut self, ty: Type) -> u64 {
            let ty_bits = ty.bits();
            debug_assert_ne!(ty_bits, 0);
            let shift = 64_u64
                .checked_sub(ty_bits.into())
                .expect("unimplemented for > 64 bits");
            u64::MAX >> shift
        }

        #[inline]
        fn ty_umin(&mut self, _ty: Type) -> u64 {
            0
        }

        #[inline]
        fn ty_umax(&mut self, ty: Type) -> u64 {
            self.ty_mask(ty)
        }

        #[inline]
        fn ty_smin(&mut self, ty: Type) -> u64 {
            let ty_bits = ty.bits();
            debug_assert_ne!(ty_bits, 0);
            let shift = 64_u64
                .checked_sub(ty_bits.into())
                .expect("unimplemented for > 64 bits");
            (i64::MIN as u64) >> shift
        }

        #[inline]
        fn ty_smax(&mut self, ty: Type) -> u64 {
            let ty_bits = ty.bits();
            debug_assert_ne!(ty_bits, 0);
            let shift = 64_u64
                .checked_sub(ty_bits.into())
                .expect("unimplemented for > 64 bits");
            (i64::MAX as u64) >> shift
        }

        fn fits_in_16(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() <= 16 && !ty.is_dynamic_vector() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn fits_in_32(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() <= 32 && !ty.is_dynamic_vector() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn lane_fits_in_32(&mut self, ty: Type) -> Option<Type> {
            if !ty.is_vector() && !ty.is_dynamic_vector() {
                None
            } else if ty.lane_type().bits() <= 32 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn fits_in_64(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() <= 64 && !ty.is_dynamic_vector() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_int_ref_scalar_64(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() <= 64 && !ty.is_float() && !ty.is_vector() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_32(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() == 32 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_64(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() == 64 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_32_or_64(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() == 32 || ty.bits() == 64 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_8_or_16(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() == 8 || ty.bits() == 16 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn int_fits_in_32(&mut self, ty: Type) -> Option<Type> {
            match ty {
                I8 | I16 | I32 => Some(ty),
                _ => None,
            }
        }

        #[inline]
        fn ty_int_ref_64(&mut self, ty: Type) -> Option<Type> {
            match ty {
                I64 | R64 => Some(ty),
                _ => None,
            }
        }

        #[inline]
        fn ty_int(&mut self, ty: Type) -> Option<Type> {
            ty.is_int().then(|| ty)
        }

        #[inline]
        fn ty_scalar_float(&mut self, ty: Type) -> Option<Type> {
            match ty {
                F32 | F64 => Some(ty),
                _ => None,
            }
        }

        #[inline]
        fn ty_float_or_vec(&mut self, ty: Type) -> Option<Type> {
            match ty {
                F32 | F64 => Some(ty),
                ty if ty.is_vector() => Some(ty),
                _ => None,
            }
        }

        fn ty_vector_float(&mut self, ty: Type) -> Option<Type> {
            if ty.is_vector() && ty.lane_type().is_float() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_vector_not_float(&mut self, ty: Type) -> Option<Type> {
            if ty.is_vector() && !ty.lane_type().is_float() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_vec64_ctor(&mut self, ty: Type) -> Option<Type> {
            if ty.is_vector() && ty.bits() == 64 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_vec64(&mut self, ty: Type) -> Option<Type> {
            if ty.is_vector() && ty.bits() == 64 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_vec128(&mut self, ty: Type) -> Option<Type> {
            if ty.is_vector() && ty.bits() == 128 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_dyn_vec64(&mut self, ty: Type) -> Option<Type> {
            if ty.is_dynamic_vector() && dynamic_to_fixed(ty).bits() == 64 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_dyn_vec128(&mut self, ty: Type) -> Option<Type> {
            if ty.is_dynamic_vector() && dynamic_to_fixed(ty).bits() == 128 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_vec64_int(&mut self, ty: Type) -> Option<Type> {
            if ty.is_vector() && ty.bits() == 64 && ty.lane_type().is_int() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_vec128_int(&mut self, ty: Type) -> Option<Type> {
            if ty.is_vector() && ty.bits() == 128 && ty.lane_type().is_int() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_addr64(&mut self, ty: Type) -> Option<Type> {
            match ty {
                I64 | R64 => Some(ty),
                _ => None,
            }
        }

        #[inline]
        fn u64_from_imm64(&mut self, imm: Imm64) -> u64 {
            imm.bits() as u64
        }

        #[inline]
        fn imm64_power_of_two(&mut self, x: Imm64) -> Option<u64> {
            let x = i64::from(x);
            let x = u64::try_from(x).ok()?;
            if x.is_power_of_two() {
                Some(x.trailing_zeros().into())
            } else {
                None
            }
        }

        #[inline]
        fn u64_from_bool(&mut self, b: bool) -> u64 {
            if b {
                u64::MAX
            } else {
                0
            }
        }

        #[inline]
        fn multi_lane(&mut self, ty: Type) -> Option<(u32, u32)> {
            if ty.lane_count() > 1 {
                Some((ty.lane_bits(), ty.lane_count()))
            } else {
                None
            }
        }

        #[inline]
        fn dynamic_lane(&mut self, ty: Type) -> Option<(u32, u32)> {
            if ty.is_dynamic_vector() {
                Some((ty.lane_bits(), ty.min_lane_count()))
            } else {
                None
            }
        }

        #[inline]
        fn dynamic_int_lane(&mut self, ty: Type) -> Option<u32> {
            if ty.is_dynamic_vector() && crate::machinst::ty_has_int_representation(ty.lane_type())
            {
                Some(ty.lane_bits())
            } else {
                None
            }
        }

        #[inline]
        fn dynamic_fp_lane(&mut self, ty: Type) -> Option<u32> {
            if ty.is_dynamic_vector()
                && crate::machinst::ty_has_float_or_vec_representation(ty.lane_type())
            {
                Some(ty.lane_bits())
            } else {
                None
            }
        }

        #[inline]
        fn ty_dyn64_int(&mut self, ty: Type) -> Option<Type> {
            if ty.is_dynamic_vector() && ty.min_bits() == 64 && ty.lane_type().is_int() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_dyn128_int(&mut self, ty: Type) -> Option<Type> {
            if ty.is_dynamic_vector() && ty.min_bits() == 128 && ty.lane_type().is_int() {
                Some(ty)
            } else {
                None
            }
        }

        fn u32_from_ieee32(&mut self, val: Ieee32) -> u32 {
            val.bits()
        }

        fn u64_from_ieee64(&mut self, val: Ieee64) -> u64 {
            val.bits()
        }

        fn u8_from_uimm8(&mut self, val: Uimm8) -> u8 {
            val
        }

        fn not_vec32x2(&mut self, ty: Type) -> Option<Type> {
            if ty.lane_bits() == 32 && ty.lane_count() == 2 {
                None
            } else {
                Some(ty)
            }
        }

        fn not_i64x2(&mut self, ty: Type) -> Option<()> {
            if ty == I64X2 {
                None
            } else {
                Some(())
            }
        }

        fn trap_code_division_by_zero(&mut self) -> TrapCode {
            TrapCode::IntegerDivisionByZero
        }

        fn trap_code_integer_overflow(&mut self) -> TrapCode {
            TrapCode::IntegerOverflow
        }

        fn trap_code_bad_conversion_to_integer(&mut self) -> TrapCode {
            TrapCode::BadConversionToInteger
        }

        fn nonzero_u64_from_imm64(&mut self, val: Imm64) -> Option<u64> {
            match val.bits() {
                0 => None,
                n => Some(n as u64),
            }
        }

        #[inline]
        fn u32_add(&mut self, a: u32, b: u32) -> u32 {
            a.wrapping_add(b)
        }

        #[inline]
        fn s32_add_fallible(&mut self, a: u32, b: u32) -> Option<u32> {
            let a = a as i32;
            let b = b as i32;
            a.checked_add(b).map(|sum| sum as u32)
        }

        #[inline]
        fn u32_nonnegative(&mut self, x: u32) -> Option<u32> {
            if (x as i32) >= 0 {
                Some(x)
            } else {
                None
            }
        }

        #[inline]
        fn u32_lteq(&mut self, a: u32, b: u32) -> Option<()> {
            if a <= b {
                Some(())
            } else {
                None
            }
        }

        #[inline]
        fn u8_lteq(&mut self, a: u8, b: u8) -> Option<()> {
            if a <= b {
                Some(())
            } else {
                None
            }
        }

        #[inline]
        fn u8_lt(&mut self, a: u8, b: u8) -> Option<()> {
            if a < b {
                Some(())
            } else {
                None
            }
        }

        #[inline]
        fn imm64(&mut self, x: u64) -> Imm64 {
            Imm64::new(x as i64)
        }

        #[inline]
        fn imm64_masked(&mut self, ty: Type, x: u64) -> Imm64 {
            Imm64::new((x & self.ty_mask(ty)) as i64)
        }

        #[inline]
        fn simm32(&mut self, x: Imm64) -> Option<u32> {
            let x64: i64 = x.into();
            let x32: i32 = x64.try_into().ok()?;
            Some(x32 as u32)
        }

        #[inline]
        fn uimm8(&mut self, x: Imm64) -> Option<u8> {
            let x64: i64 = x.into();
            let x8: u8 = x64.try_into().ok()?;
            Some(x8)
        }

        #[inline]
        fn offset32(&mut self, x: Offset32) -> u32 {
            let x: i32 = x.into();
            x as u32
        }

        #[inline]
        fn u8_and(&mut self, a: u8, b: u8) -> u8 {
            a & b
        }

        #[inline]
        fn lane_type(&mut self, ty: Type) -> Type {
            ty.lane_type()
        }

        #[inline]
        fn offset32_to_u32(&mut self, offset: Offset32) -> u32 {
            let offset: i32 = offset.into();
            offset as u32
        }

        #[inline]
        fn u32_to_offset32(&mut self, offset: u32) -> Offset32 {
            Offset32::new(offset as i32)
        }

        fn range(&mut self, start: usize, end: usize) -> Range {
            (start, end)
        }

        fn range_view(&mut self, (start, end): Range) -> RangeView {
            if start >= end {
                RangeView::Empty
            } else {
                RangeView::NonEmpty {
                    index: start,
                    rest: (start + 1, end),
                }
            }
        }

        #[inline]
        fn mem_flags_trusted(&mut self) -> MemFlags {
            MemFlags::trusted()
        }

        #[inline]
        fn intcc_unsigned(&mut self, x: &IntCC) -> IntCC {
            x.unsigned()
        }

        #[inline]
        fn signed_cond_code(&mut self, cc: &condcodes::IntCC) -> Option<condcodes::IntCC> {
            match cc {
                IntCC::Equal
                | IntCC::UnsignedGreaterThanOrEqual
                | IntCC::UnsignedGreaterThan
                | IntCC::UnsignedLessThanOrEqual
                | IntCC::UnsignedLessThan
                | IntCC::NotEqual => None,
                IntCC::SignedGreaterThanOrEqual
                | IntCC::SignedGreaterThan
                | IntCC::SignedLessThanOrEqual
                | IntCC::SignedLessThan => Some(*cc),
            }
        }

        #[inline]
        fn intcc_reverse(&mut self, cc: &IntCC) -> IntCC {
            cc.reverse()
        }

        #[inline]
        fn intcc_inverse(&mut self, cc: &IntCC) -> IntCC {
            cc.inverse()
        }

        #[inline]
        fn floatcc_reverse(&mut self, cc: &FloatCC) -> FloatCC {
            cc.reverse()
        }

        #[inline]
        fn floatcc_inverse(&mut self, cc: &FloatCC) -> FloatCC {
            cc.inverse()
        }

        fn floatcc_unordered(&mut self, cc: &FloatCC) -> bool {
            match *cc {
                FloatCC::Unordered
                | FloatCC::UnorderedOrEqual
                | FloatCC::UnorderedOrLessThan
                | FloatCC::UnorderedOrLessThanOrEqual
                | FloatCC::UnorderedOrGreaterThan
                | FloatCC::UnorderedOrGreaterThanOrEqual => true,
                _ => false,
            }
        }

        #[inline]
        fn unpack_value_array_2(&mut self, arr: &ValueArray2) -> (Value, Value) {
            let [a, b] = *arr;
            (a, b)
        }

        #[inline]
        fn pack_value_array_2(&mut self, a: Value, b: Value) -> ValueArray2 {
            [a, b]
        }

        #[inline]
        fn unpack_value_array_3(&mut self, arr: &ValueArray3) -> (Value, Value, Value) {
            let [a, b, c] = *arr;
            (a, b, c)
        }

        #[inline]
        fn pack_value_array_3(&mut self, a: Value, b: Value, c: Value) -> ValueArray3 {
            [a, b, c]
        }

        #[inline]
        fn unpack_block_array_2(&mut self, arr: &BlockArray2) -> (BlockCall, BlockCall) {
            let [a, b] = *arr;
            (a, b)
        }

        #[inline]
        fn pack_block_array_2(&mut self, a: BlockCall, b: BlockCall) -> BlockArray2 {
            [a, b]
        }

        fn u128_as_u64(&mut self, val: u128) -> Option<u64> {
            u64::try_from(val).ok()
        }

        fn u64_as_u32(&mut self, val: u64) -> Option<u32> {
            u32::try_from(val).ok()
        }
    };
}
