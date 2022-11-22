use windows::Win32::Foundation::{POINT, RECT};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Position<T, Coord> {
    pub x: T,
    pub y: T,
    #[cfg_attr(feature = "serde", serde(skip))]
    _coord: std::marker::PhantomData<Coord>,
}

impl<T, Coord> Position<T, Coord> {
    #[inline]
    pub const fn new(x: T, y: T) -> Self {
        Self {
            x,
            y,
            _coord: std::marker::PhantomData,
        }
    }
}

impl<T, Coord> Position<T, Coord>
where
    T: num::NumCast,
{
    #[inline]
    pub fn cast<U>(self) -> Option<Position<U, Coord>>
    where
        U: num::NumCast,
    {
        Some(Position::new(num::cast(self.x)?, num::cast(self.y)?))
    }
}

impl<T, Coord> From<(T, T)> for Position<T, Coord> {
    #[inline]
    fn from(src: (T, T)) -> Self {
        Position::new(src.0, src.1)
    }
}

impl<T, Coord> From<[T; 2]> for Position<T, Coord>
where
    T: Copy,
{
    #[inline]
    fn from(src: [T; 2]) -> Self {
        Position::new(src[0], src[1])
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Size<T, Coord> {
    pub width: T,
    pub height: T,
    #[cfg_attr(feature = "serde", serde(skip))]
    _coord: std::marker::PhantomData<Coord>,
}

impl<T, Coord> Size<T, Coord> {
    #[inline]
    pub const fn new(width: T, height: T) -> Self {
        Self {
            width,
            height,
            _coord: std::marker::PhantomData,
        }
    }
}

impl<T, Coord> Size<T, Coord>
where
    T: num::NumCast,
{
    #[inline]
    pub fn cast<U>(self) -> Option<Size<U, Coord>>
    where
        U: num::NumCast,
    {
        Some(Size::new(num::cast(self.width)?, num::cast(self.height)?))
    }
}

impl<T, Coord> From<(T, T)> for Size<T, Coord> {
    #[inline]
    fn from(src: (T, T)) -> Self {
        Size::new(src.0, src.1)
    }
}

impl<T, Coord> From<[T; 2]> for Size<T, Coord>
where
    T: Copy,
{
    #[inline]
    fn from(src: [T; 2]) -> Self {
        Size::new(src[0], src[1])
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rect<T, Coord> {
    pub position: Position<T, Coord>,
    pub size: Size<T, Coord>,
}

impl<T, Coord> Rect<T, Coord> {
    #[inline]
    pub fn new(position: impl Into<Position<T, Coord>>, size: impl Into<Size<T, Coord>>) -> Self {
        Self {
            position: position.into(),
            size: size.into(),
        }
    }
}

impl<T, Coord> Rect<T, Coord>
where
    T: Copy + num::Num,
    Coord: Copy,
{
    #[inline]
    pub fn from_positions(
        lt: impl Into<Position<T, Coord>>,
        rb: impl Into<Position<T, Coord>>,
    ) -> Self {
        let lt = lt.into();
        let rb = rb.into();
        Self {
            position: lt,
            size: (rb.x - lt.x, rb.y - lt.y).into(),
        }
    }

    #[inline]
    pub fn endpoint(&self) -> Position<T, Coord> {
        Position::new(
            self.position.x + self.size.width,
            self.position.y + self.size.height,
        )
    }
}

impl<T, Coord> Rect<T, Coord>
where
    T: num::NumCast,
{
    #[inline]
    pub fn cast<U>(self) -> Option<Rect<U, Coord>>
    where
        U: num::NumCast,
    {
        Some(Rect::new(
            self.position.cast::<U>()?,
            self.size.cast::<U>()?,
        ))
    }
}

pub const DEFAULT_DPI: u32 = 96;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Physical;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Logical;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Screen;

pub type PhysicalPosition<T> = Position<T, Physical>;
pub type LogicalPosition<T> = Position<T, Logical>;
pub type ScreenPosition = Position<i32, Screen>;

pub type PhysicalSize<T> = Size<T, Physical>;
pub type LogicalSize<T> = Size<T, Logical>;

pub type PhysicalRect<T> = Rect<T, Physical>;
pub type LogicalRect<T> = Rect<T, Logical>;

fn to_logical_value<T>(a: T, dpi: T) -> T
where
    T: num::Num + num::NumCast,
{
    a * num::cast(DEFAULT_DPI).unwrap() / dpi
}

fn to_physical_value<T>(a: T, dpi: T) -> T
where
    T: num::Num + num::NumCast,
{
    a * dpi / num::cast(DEFAULT_DPI).unwrap()
}

pub trait ToLogical<T> {
    type Output<U>;

    fn to_logical(&self, dpi: T) -> Self::Output<T>;
}

impl<T> ToLogical<T> for LogicalPosition<T>
where
    T: Copy,
{
    type Output<U> = LogicalPosition<U>;

    #[inline]
    fn to_logical(&self, _dpi: T) -> Self::Output<T> {
        *self
    }
}

impl<T> ToLogical<T> for PhysicalPosition<T>
where
    T: num::Num + num::NumCast + Copy,
{
    type Output<U> = LogicalPosition<U>;

    #[inline]
    fn to_logical(&self, dpi: T) -> Self::Output<T> {
        Position::new(to_logical_value(self.x, dpi), to_logical_value(self.y, dpi))
    }
}

impl<T> ToLogical<T> for LogicalSize<T>
where
    T: Copy,
{
    type Output<U> = LogicalSize<U>;

    #[inline]
    fn to_logical(&self, _dpi: T) -> Self::Output<T> {
        *self
    }
}

impl<T> ToLogical<T> for PhysicalSize<T>
where
    T: num::Num + num::NumCast + Copy,
{
    type Output<U> = LogicalSize<U>;

    #[inline]
    fn to_logical(&self, dpi: T) -> Self::Output<T> {
        Size::new(
            to_logical_value(self.width, dpi),
            to_logical_value(self.height, dpi),
        )
    }
}

impl<T> ToLogical<T> for LogicalRect<T>
where
    T: Copy,
{
    type Output<U> = LogicalRect<U>;

    #[inline]
    fn to_logical(&self, _dpi: T) -> Self::Output<T> {
        *self
    }
}

impl<T> ToLogical<T> for PhysicalRect<T>
where
    T: num::Num + num::NumCast + Copy,
{
    type Output<U> = LogicalRect<T>;

    #[inline]
    fn to_logical(&self, dpi: T) -> Self::Output<T> {
        Rect::new(self.position.to_logical(dpi), self.size.to_logical(dpi))
    }
}

pub trait ToPhysical<T> {
    type Output<U>;

    fn to_physical(&self, dpi: T) -> Self::Output<T>;
}

impl<T> ToPhysical<T> for LogicalPosition<T>
where
    T: num::Num + num::NumCast + Copy,
{
    type Output<U> = PhysicalPosition<U>;

    #[inline]
    fn to_physical(&self, dpi: T) -> Self::Output<T> {
        Position::new(
            to_physical_value(self.x, dpi),
            to_physical_value(self.y, dpi),
        )
    }
}

impl<T> ToPhysical<T> for PhysicalPosition<T>
where
    T: Copy,
{
    type Output<U> = PhysicalPosition<U>;

    #[inline]
    fn to_physical(&self, _dpi: T) -> Self::Output<T> {
        *self
    }
}

impl<T> ToPhysical<T> for LogicalSize<T>
where
    T: num::Num + num::NumCast + Copy,
{
    type Output<U> = PhysicalSize<U>;

    #[inline]
    fn to_physical(&self, dpi: T) -> Self::Output<T> {
        Size::new(
            to_physical_value(self.width, dpi),
            to_physical_value(self.height, dpi),
        )
    }
}

impl<T> ToPhysical<T> for PhysicalSize<T>
where
    T: Copy,
{
    type Output<U> = PhysicalSize<U>;

    #[inline]
    fn to_physical(&self, _dpi: T) -> Self::Output<T> {
        *self
    }
}

impl<T> ToPhysical<T> for LogicalRect<T>
where
    T: num::Num + num::NumCast + Copy,
{
    type Output<U> = PhysicalRect<U>;

    #[inline]
    fn to_physical(&self, dpi: T) -> Self::Output<T> {
        Rect::new(self.position.to_physical(dpi), self.size.to_physical(dpi))
    }
}

impl<T> ToPhysical<T> for PhysicalRect<T>
where
    T: Copy,
{
    type Output<U> = PhysicalRect<U>;

    #[inline]
    fn to_physical(&self, _dpi: T) -> Self::Output<T> {
        *self
    }
}

impl From<POINT> for PhysicalPosition<i32> {
    #[inline]
    fn from(src: POINT) -> Self {
        Self::new(src.x, src.y)
    }
}

impl From<RECT> for PhysicalRect<i32> {
    #[inline]
    fn from(src: RECT) -> Self {
        Self::from_positions((src.left, src.top), (src.right, src.bottom))
    }
}

impl From<PhysicalRect<i32>> for RECT {
    #[inline]
    fn from(src: PhysicalRect<i32>) -> Self {
        let rb = src.endpoint();
        RECT {
            left: src.position.x,
            top: src.position.y,
            right: rb.x,
            bottom: rb.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_to_logical_position() {
        let src = LogicalPosition::new(126, 256);
        let dest = src.to_logical(DEFAULT_DPI * 2);
        assert!(src.x == dest.x);
        assert!(src.y == dest.y);
    }

    #[test]
    fn logical_to_physical_position() {
        let src = LogicalPosition::new(128, 256);
        let dest = src.to_physical(DEFAULT_DPI * 2);
        assert!(src.x * 2 == dest.x);
        assert!(src.y * 2 == dest.y);
    }

    #[test]
    fn physical_to_logical_position() {
        let src = PhysicalPosition::new(128, 256);
        let dest = src.to_logical(DEFAULT_DPI * 2);
        assert!(src.x == dest.x * 2);
        assert!(src.y == dest.y * 2);
    }

    #[test]
    fn physical_to_physical_position() {
        let src = PhysicalPosition::new(128, 256);
        let dest = src.to_physical(DEFAULT_DPI * 2);
        assert!(src.x == dest.x);
        assert!(src.y == dest.y);
    }

    #[test]
    fn logical_to_logical_size() {
        let src = LogicalSize::new(126, 256);
        let dest = src.to_logical(DEFAULT_DPI * 2);
        assert!(src.width == dest.width);
        assert!(src.height == dest.height);
    }

    #[test]
    fn logical_to_physical_size() {
        let src = LogicalSize::new(128, 256);
        let dest = src.to_physical(DEFAULT_DPI * 2);
        assert!(src.width * 2 == dest.width);
        assert!(src.height * 2 == dest.height);
    }

    #[test]
    fn physical_to_logical_size() {
        let src = PhysicalSize::new(128, 256);
        let dest = src.to_logical(DEFAULT_DPI * 2);
        assert!(src.width == dest.width * 2);
        assert!(src.height == dest.height * 2);
    }

    #[test]
    fn physical_to_physical_size() {
        let src = PhysicalSize::new(128, 256);
        let dest = src.to_physical(DEFAULT_DPI * 2);
        assert!(src.width == dest.width);
        assert!(src.height == dest.height);
    }

    #[test]
    fn logical_to_logical_rect() {
        let src = LogicalRect::new((2, 4), (100, 200));
        let dest = src.to_logical(DEFAULT_DPI * 2);
        assert!(src == dest)
    }

    #[test]
    fn logical_to_physical_rect() {
        let src = LogicalRect::new((2, 4), (100, 200));
        let dest = src.to_physical(DEFAULT_DPI * 2);
        assert!(src.position.x * 2 == dest.position.x);
        assert!(src.position.y * 2 == dest.position.y);
        assert!(src.size.width * 2 == dest.size.width);
        assert!(src.size.height * 2 == dest.size.height);
    }

    #[test]
    fn physical_to_logical_rect() {
        let src = PhysicalRect::new((2, 4), (100, 200));
        let dest = src.to_logical(DEFAULT_DPI * 2);
        assert!(src.position.x == dest.position.x * 2);
        assert!(src.position.y == dest.position.y * 2);
        assert!(src.size.width == dest.size.width * 2);
        assert!(src.size.height == dest.size.height * 2);
    }

    #[test]
    fn phsyical_to_physical_rect() {
        let src = PhysicalRect::new((2, 4), (100, 200));
        let dest = src.to_physical(DEFAULT_DPI * 2);
        assert!(src == dest)
    }
}
