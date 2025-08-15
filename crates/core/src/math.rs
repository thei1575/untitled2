//! Math utilities shared across crates.

/// 3D integer vector for chunk and block coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IVec3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl IVec3 {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };
    pub const X: Self = Self { x: 1, y: 0, z: 0 };
    pub const Y: Self = Self { x: 0, y: 1, z: 0 };
    pub const Z: Self = Self { x: 0, y: 0, z: 1 };

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn dot(self, other: Self) -> i32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn length_squared(self) -> i32 {
        self.dot(self)
    }

    pub fn abs(self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }

    pub fn min(self, other: Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }

    pub fn max(self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }

    pub fn clamp(self, min: Self, max: Self) -> Self {
        self.max(min).min(max)
    }
}

impl std::ops::Add for IVec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Sub for IVec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::Mul<i32> for IVec3 {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl std::ops::Div<i32> for IVec3 {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

/// Chunk size constants.
pub const CHUNK_SIZE: i32 = 16;
pub const CHUNK_HEIGHT: i32 = 256;
pub const CHUNK_VOLUME: usize = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT) as usize;

/// Convert world coordinates to chunk coordinates.
pub fn world_to_chunk(world_pos: IVec3) -> IVec3 {
    IVec3::new(
        world_pos.x.div_euclid(CHUNK_SIZE),
        0, // Chunks span full height
        world_pos.z.div_euclid(CHUNK_SIZE),
    )
}

/// Convert world coordinates to local block coordinates within a chunk.
pub fn world_to_local(world_pos: IVec3) -> IVec3 {
    IVec3::new(
        world_pos.x.rem_euclid(CHUNK_SIZE),
        world_pos.y,
        world_pos.z.rem_euclid(CHUNK_SIZE),
    )
}

/// Convert chunk coordinates and local coordinates to world coordinates.
pub fn chunk_local_to_world(chunk_pos: IVec3, local_pos: IVec3) -> IVec3 {
    IVec3::new(
        chunk_pos.x * CHUNK_SIZE + local_pos.x,
        local_pos.y,
        chunk_pos.z * CHUNK_SIZE + local_pos.z,
    )
}

/// Convert 3D local coordinates to 1D array index.
pub fn local_to_index(local_pos: IVec3) -> Option<usize> {
    if local_pos.x < 0 || local_pos.x >= CHUNK_SIZE
        || local_pos.y < 0 || local_pos.y >= CHUNK_HEIGHT
        || local_pos.z < 0 || local_pos.z >= CHUNK_SIZE
    {
        return None;
    }

    let index = (local_pos.y * CHUNK_SIZE * CHUNK_SIZE + local_pos.z * CHUNK_SIZE + local_pos.x) as usize;
    Some(index)
}

/// Convert 1D array index to 3D local coordinates.
pub fn index_to_local(index: usize) -> Option<IVec3> {
    if index >= CHUNK_VOLUME {
        return None;
    }

    let index = index as i32;
    let y = index / (CHUNK_SIZE * CHUNK_SIZE);
    let remainder = index % (CHUNK_SIZE * CHUNK_SIZE);
    let z = remainder / CHUNK_SIZE;
    let x = remainder % CHUNK_SIZE;

    Some(IVec3::new(x, y, z))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ivec3_operations() {
        let a = IVec3::new(1, 2, 3);
        let b = IVec3::new(4, 5, 6);

        assert_eq!(a + b, IVec3::new(5, 7, 9));
        assert_eq!(b - a, IVec3::new(3, 3, 3));
        assert_eq!(a * 2, IVec3::new(2, 4, 6));
        assert_eq!(a / 2, IVec3::new(0, 1, 1)); // Integer division

        assert_eq!(a.dot(b), 32); // 1*4 + 2*5 + 3*6
        assert_eq!(a.length_squared(), 14); // 1 + 4 + 9
    }

    #[test]
    fn test_coordinate_conversions() {
        // Test world to chunk conversion
        assert_eq!(world_to_chunk(IVec3::new(0, 0, 0)), IVec3::new(0, 0, 0));
        assert_eq!(world_to_chunk(IVec3::new(15, 100, 15)), IVec3::new(0, 0, 0));
        assert_eq!(world_to_chunk(IVec3::new(16, 50, 16)), IVec3::new(1, 0, 1));
        assert_eq!(world_to_chunk(IVec3::new(-1, 0, -1)), IVec3::new(-1, 0, -1));
        assert_eq!(world_to_chunk(IVec3::new(-16, 0, -16)), IVec3::new(-1, 0, -1));
        assert_eq!(world_to_chunk(IVec3::new(-17, 0, -17)), IVec3::new(-2, 0, -2));

        // Test world to local conversion
        assert_eq!(world_to_local(IVec3::new(0, 100, 0)), IVec3::new(0, 100, 0));
        assert_eq!(world_to_local(IVec3::new(15, 50, 15)), IVec3::new(15, 50, 15));
        assert_eq!(world_to_local(IVec3::new(16, 25, 16)), IVec3::new(0, 25, 0));
        assert_eq!(world_to_local(IVec3::new(-1, 10, -1)), IVec3::new(15, 10, 15));

        // Test round-trip conversion
        let world_pos = IVec3::new(123, 45, -67);
        let chunk_pos = world_to_chunk(world_pos);
        let local_pos = world_to_local(world_pos);
        let reconstructed = chunk_local_to_world(chunk_pos, local_pos);
        assert_eq!(world_pos, reconstructed);
    }

    #[test]
    fn test_index_conversions() {
        // Test valid coordinates
        let pos = IVec3::new(5, 10, 7);
        let index = local_to_index(pos).unwrap();
        let reconstructed = index_to_local(index).unwrap();
        assert_eq!(pos, reconstructed);

        // Test bounds checking
        assert!(local_to_index(IVec3::new(-1, 0, 0)).is_none());
        assert!(local_to_index(IVec3::new(CHUNK_SIZE, 0, 0)).is_none());
        assert!(local_to_index(IVec3::new(0, -1, 0)).is_none());
        assert!(local_to_index(IVec3::new(0, CHUNK_HEIGHT, 0)).is_none());
        assert!(local_to_index(IVec3::new(0, 0, CHUNK_SIZE)).is_none());

        // Test index bounds
        assert!(index_to_local(CHUNK_VOLUME).is_none());
        assert!(index_to_local(CHUNK_VOLUME + 1).is_none());

        // Test corner cases
        assert_eq!(local_to_index(IVec3::ZERO).unwrap(), 0);
        assert_eq!(index_to_local(0).unwrap(), IVec3::ZERO);

        let max_pos = IVec3::new(CHUNK_SIZE - 1, CHUNK_HEIGHT - 1, CHUNK_SIZE - 1);
        let max_index = local_to_index(max_pos).unwrap();
        assert_eq!(max_index, CHUNK_VOLUME - 1);
        assert_eq!(index_to_local(max_index).unwrap(), max_pos);
    }
}
