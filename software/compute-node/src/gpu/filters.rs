use std::ops;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum GpuFilter {
    Sobel,
    Gaussian2dBlur20,
    GaussianHBlur,
    GaussianVBlur,
    BoxHBlur,
    BoxVBlur,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum GpuFilterSequence {
    Apply(GpuFilter),
    Chain((GpuFilter, Box<GpuFilterSequence>)),
    Repeat(u32, Box<GpuFilterSequence>),
}

impl GpuFilterSequence {
    pub fn to_vec(&self) -> Vec<GpuFilter> {
        match self {
            GpuFilterSequence::Apply(filter) => vec![filter.clone()],
            GpuFilterSequence::Chain((filter, chain)) => {
                let mut v = vec![filter.clone()];
                v.extend(chain.to_vec());
                v
            }
            GpuFilterSequence::Repeat(n, chain) => {
                let mut v = Vec::new();
                for _ in 0..*n {
                    v.extend(chain.to_vec());
                }
                v
            }
        }
    }
}

impl ops::Add<GpuFilterSequence> for GpuFilter {
    type Output = GpuFilterSequence;

    fn add(self, rhs: GpuFilterSequence) -> GpuFilterSequence {
        GpuFilterSequence::Chain((self, Box::new(rhs)))
    }
}

impl ops::Add<GpuFilter> for GpuFilter {
    type Output = GpuFilterSequence;

    fn add(self, rhs: GpuFilter) -> GpuFilterSequence {
        GpuFilterSequence::Chain((self, Box::new(GpuFilterSequence::Apply(rhs))))
    }
}

impl ops::Add<GpuFilter> for GpuFilterSequence {
    type Output = GpuFilterSequence;

    fn add(self, rhs: GpuFilter) -> GpuFilterSequence {
        GpuFilterSequence::Chain((rhs, Box::new(self)))
    }
}

impl ops::Mul<u32> for GpuFilterSequence {
    type Output = GpuFilterSequence;

    fn mul(self, rhs: u32) -> GpuFilterSequence {
        GpuFilterSequence::Repeat(rhs, Box::new(self))
    }
}

impl ops::Mul<u32> for GpuFilter {
    type Output = GpuFilterSequence;

    fn mul(self, rhs: u32) -> GpuFilterSequence {
        GpuFilterSequence::Repeat(rhs, Box::new(GpuFilterSequence::Apply(self)))
    }
}

impl From<GpuFilter> for GpuFilterSequence {
    fn from(filter: GpuFilter) -> Self {
        GpuFilterSequence::Apply(filter)
    }
}

impl std::fmt::Display for GpuFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuFilter::Sobel => write!(f, "sobel"),
            GpuFilter::Gaussian2dBlur20 => write!(f, "gaussian_2d_blur_20"),
            GpuFilter::GaussianHBlur => write!(f, "gaussian_hblur"),
            GpuFilter::GaussianVBlur => write!(f, "gaussian_vblur"),
            GpuFilter::BoxHBlur => write!(f, "box_hblur"),
            GpuFilter::BoxVBlur => write!(f, "box_vblur"),
        }
    }
}
