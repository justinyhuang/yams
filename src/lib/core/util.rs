use std::mem::transmute;

pub fn write_be_u16_into_f32(src: &[u16], dst: &mut [f32])
{
    let num_f32 = src.len() / 2;

    for idx in 0..num_f32 {
        let uint32: u32 = src[idx * 2] as u32 | ((src[idx * 2 + 1] as u32) << 16);
        unsafe {
            dst[idx] = transmute(uint32);
        }
    }
}

pub fn write_be_u16_into_f64(src: &[u16], dst: &mut [f64])
{
    let num_f64 = src.len() / 4;

    for idx in 0..num_f64 {
        let uint64: u64 = src[idx * 4] as u64 |
                          ((src[idx * 4 + 1] as u64) << 16) |
                          ((src[idx * 4 + 2] as u64) << 32) |
                          ((src[idx * 4 + 3] as u64) << 48);
        unsafe {
            dst[idx] = transmute(uint64);
        }
    }
}

pub fn write_be_f32_into_u16(src: f32, dst: &mut [u16])
{
    let uint32: u32 = unsafe {
        transmute(src)
    };
    dst[0] = (uint32 & 0xFFFF) as u16;
    dst[1] = (uint32 >> 16) as u16;
}

pub fn write_be_f64_into_u16(src: f64, dst: &mut [u16])
{
    let uint64: u64 = unsafe {
        transmute(src)
    };
    dst[0] = (uint64 & 0xFFFF) as u16;
    dst[1] = ((uint64 & 0xFFFF0000) >> 16) as u16;
    dst[2] = ((uint64 & 0xFFFF00000000) >> 32) as u16;
    dst[3] = ((uint64 & 0xFFFF000000000000) >> 48) as u16;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use float_cmp::assert_approx_eq;

    #[test]
    fn given_2_u16_then_get_correct_f32()
    {
        let src = [0x0e56, 0x4049];
        let mut dst = [0.0 as f32; 1];
        write_be_u16_into_f32(&src, &mut dst);
        assert_approx_eq!(f32, 3.1415_f32, dst[0]);
    }

    #[test]
    fn given4_u16_then_get_correct_f64()
    {
        let src = [0x2D18, 0x5444, 0x21FB, 0x4009];
        let mut dst = [0.0 as f64; 1];
        write_be_u16_into_f64(&src, &mut dst);
        assert_approx_eq!(f64, 3.141592653589793_f64, dst[0]);
    }

    #[test]
    fn given_f32_then_get_correct_2_u16()
    {
        let src = 3.1415_f32;
        let mut dst = [0_u16; 2];
        write_be_f32_into_u16(src, &mut dst);
        assert_eq!([0x0e56, 0x4049], dst);
    }

    #[test]
    fn given_f64_then_get_correct_4_u16()
    {
        let src = 3.141592653589793_f64;
        let mut dst = [0_u16; 4];
        write_be_f64_into_u16(src, &mut dst);
        assert_eq!([0x2D18, 0x5444, 0x21FB, 0x4009], dst);
    }
}
