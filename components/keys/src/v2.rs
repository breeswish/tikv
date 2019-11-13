// Copyright 2019 TiKV Project Authors. Licensed under Apache-2.0.

use std::borrow::Borrow;
use std::fmt::{self, Debug, Display};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use super::types::Key;
use codec::byte::MemComparableByteCodec;
use codec::prelude::*;

pub trait KeyLike:
    Send + Sync + Debug + Display + Hash + PartialEq + Eq + PartialOrd + Ord
{
}

pub trait PhysicalKey: Sized + Clone + KeyLike + NumberEncoder + BufferWriter + Deref {
    const PHYSICAL_PREFIX: &'static [u8];
    type Slice: PhysicalKeySlice<OwnedKey = Self> + ?Sized;

    /// Only used for `PhysicalKey` implementations. Not intended to be used elsewhere.
    #[doc(hidden)]
    fn _new_from_vec(vec: Vec<u8>) -> Self;

    /// Only used for `PhysicalKey` implementations. Not intended to be used elsewhere.
    #[doc(hidden)]
    fn _vec_ref(&self) -> &Vec<u8>;

    /// Only used for `PhysicalKey` implementations. Not intended to be used elsewhere.
    #[doc(hidden)]
    fn _vec_mut(&mut self) -> &mut Vec<u8>;

    /// Only used for `PhysicalKey` implementations. Not intended to be used elsewhere.
    #[doc(hidden)]
    fn _into_vec(self) -> Vec<u8>;

    #[inline(never)]
    fn into_physical_vec(self) -> Vec<u8> {
        self._into_vec()
    }

    #[inline(never)]
    fn as_physical_std_slice(&self) -> &[u8] {
        self._vec_ref().as_slice()
    }

    #[inline(never)]
    fn as_physical_slice(&self) -> &Self::Slice {
        Self::Slice::from_physical_std_slice(self.as_physical_std_slice())
    }

    #[inline(never)]
    fn as_physical_slice_without_ts(&self) -> &Self::Slice {
        self.as_physical_slice().as_physical_slice_without_ts()
    }

    #[inline(never)]
    fn as_logical_slice(&self) -> &LogicalKeySlice {
        self.as_physical_slice().as_logical_slice()
    }

    #[inline(never)]
    fn as_logical_slice_without_ts(&self) -> &LogicalKeySlice {
        self.as_physical_slice().as_logical_slice_without_ts()
    }

    #[inline(never)]
    fn as_logical_std_slice(&self) -> &[u8] {
        self.as_logical_slice().as_std_slice()
    }

    #[inline(never)]
    fn from_physical_vec(pk: Vec<u8>) -> Self {
        assert!(pk.starts_with(Self::PHYSICAL_PREFIX));
        Self::_new_from_vec(pk)
    }

    #[inline(never)]
    fn alloc_from_physical_std_slice(pk: &[u8]) -> Self {
        let mut v = Vec::with_capacity(pk.len() + 8);
        v.extend_from_slice(pk);
        Self::from_physical_vec(v)
    }

    #[inline(never)]
    fn alloc_from_physical_slice(pk: &Self::Slice) -> Self {
        Self::alloc_from_physical_std_slice(pk.as_physical_std_slice())
    }

    #[inline(never)]
    fn alloc_with_logical_capacity(capacity: usize) -> Self {
        let mut vec = Vec::with_capacity(Self::PHYSICAL_PREFIX.len() + capacity + 8);
        vec.extend_from_slice(Self::PHYSICAL_PREFIX);
        Self::_new_from_vec(vec)
    }

    #[inline(never)]
    fn alloc_new() -> Self {
        // Note: 40 is a size suitable for TiDB payload (without ts suffix).
        Self::alloc_with_logical_capacity(40)
    }

    #[inline(never)]
    fn alloc_from_logical_std_slice(lk: &[u8]) -> Self {
        let mut physical_key = Self::alloc_with_logical_capacity(lk.len());
        physical_key.write_bytes(lk).unwrap();
        physical_key
    }

    #[inline(never)]
    fn alloc_from_logical_slice(lk: &LogicalKeySlice) -> Self {
        Self::alloc_from_logical_std_slice(lk.as_std_slice())
    }

    fn copy_from_logical_vec(mut lk: Vec<u8>) -> Self {
        if Self::PHYSICAL_PREFIX.is_empty() {
            Self::_new_from_vec(lk)
        } else {
            use std::ptr::copy;
            unsafe {
                let len = lk.len();
                let prefix_len = Self::PHYSICAL_PREFIX.len();
                lk.reserve(prefix_len + 8);
                copy(lk.as_ptr(), lk.as_mut_ptr().add(prefix_len), len);
                copy(Self::PHYSICAL_PREFIX.as_ptr(), lk.as_mut_ptr(), prefix_len);
                lk.set_len(len + prefix_len);
            }
            Self::_new_from_vec(lk)
        }
    }

    // FIXME: This is a MVCC knowledge.
    #[inline(never)]
    fn alloc_from_user_std_slice(uk: &[u8]) -> Self {
        let mut key: Self =
            Self::alloc_with_logical_capacity(MemComparableByteCodec::encoded_len(uk.len()));
        // 8 for timestamp
        key.write_comparable_bytes(uk).unwrap();
        key
    }

    // FIXME: This is a MVCC knowledge.
    // FIXME: Use in place encoding to avoid allocation
    #[inline(never)]
    fn alloc_from_user_vec(uk: Vec<u8>) -> Self {
        Self::alloc_from_user_std_slice(uk.as_slice())
    }

    #[inline(never)]
    fn physical_len(&self) -> usize {
        self.as_physical_slice().len()
    }

    #[inline(never)]
    fn logical_len(&self) -> usize {
        self.as_logical_slice().len()
    }

    // FIXME: This is a MVCC knowledge.
    #[inline(never)]
    fn append_ts(&mut self, ts: u64) {
        self.write_u64_desc(ts).unwrap();
    }

    // FIXME: This is a MVCC knowledge.
    #[inline(never)]
    fn shrink_ts(&mut self) {
        let len = self._vec_ref().len();
        self._vec_mut().truncate(len - 8);
    }

    #[inline(never)]
    fn with_ts_temporarily(&mut self, ts: u64) -> PhysicalKeyTsGuard<'_, Self> {
        PhysicalKeyTsGuard::new(self, ts)
    }

    #[inline(never)]
    fn get_ts(&self) -> u64 {
        self.as_physical_slice().get_ts()
    }

    #[inline(never)]
    fn reset_from_physical_slice(&mut self, pk: &Self::Slice) {
        self._vec_mut().clear();
        self.write_bytes(pk.as_physical_std_slice()).unwrap();
    }

    #[inline(never)]
    fn reset_from_logical_std_slice(&mut self, lk: &[u8]) {
        self._vec_mut().truncate(Self::PHYSICAL_PREFIX.len());
        self.write_bytes(lk).unwrap();
    }

    #[inline(never)]
    fn reset_from_logical_slice(&mut self, lk: &LogicalKeySlice) {
        self.reset_from_logical_std_slice(lk.as_std_slice())
    }

    // FIXME: This is a MVCC knowledge.
    #[inline(never)]
    fn reset_from_user_std_slice(&mut self, uk: &[u8]) {
        self._vec_mut().truncate(Self::PHYSICAL_PREFIX.len());
        self.write_comparable_bytes(uk).unwrap();
    }
}

pub struct PhysicalKeyTsGuard<'a, Key: PhysicalKey> {
    key: &'a mut Key,
}

impl<'a, Key: PhysicalKey> PhysicalKeyTsGuard<'a, Key> {
    #[inline(never)]
    pub fn new(key: &'a mut Key, ts: u64) -> Self {
        key.append_ts(ts);
        Self { key }
    }
}

impl<'a, Key: PhysicalKey> Deref for PhysicalKeyTsGuard<'a, Key> {
    type Target = Key;

    #[inline(never)]
    fn deref(&self) -> &Self::Target {
        self.key
    }
}

impl<'a, Key: PhysicalKey> Drop for PhysicalKeyTsGuard<'a, Key> {
    #[inline(never)]
    fn drop(&mut self) {
        self.key.shrink_ts()
    }
}

pub trait PhysicalKeySlice: KeyLike + ToPhysicalKeySlice<Self> {
    type OwnedKey: PhysicalKey<Slice = Self>;

    // TODO: Only to support `impl Key for ToPhysicalKeySlice<T>`. To be removed.
    type LegacyKeySliceOwner;

    // TODO: Only to support `impl Key for ToPhysicalKeySlice<T>`. To be removed.
    fn from_legacy_key(key: &Key) -> PKContainer<'_, Self::LegacyKeySliceOwner, Self>;

    fn as_physical_std_slice(&self) -> &[u8];

    fn from_physical_std_slice(s: &[u8]) -> &Self;

    fn as_logical_slice(&self) -> &LogicalKeySlice;

    #[inline(never)]
    fn as_logical_slice_without_ts(&self) -> &LogicalKeySlice {
        self.as_logical_slice().as_physical_slice_without_ts()
    }

    #[inline(never)]
    fn as_logical_std_slice(&self) -> &[u8] {
        self.as_logical_slice().as_std_slice()
    }

    #[inline(never)]
    fn as_physical_slice_without_ts(&self) -> &Self {
        let s = self.as_physical_std_slice();
        Self::from_physical_std_slice(&s[..s.len() - 8])
    }

    #[inline(never)]
    fn alloc_to_physical_key(&self) -> Self::OwnedKey {
        Self::OwnedKey::alloc_from_physical_slice(self)
    }

    #[inline(never)]
    fn len(&self) -> usize {
        self.as_physical_std_slice().len()
    }

    #[inline(never)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(never)]
    fn get_ts(&self) -> u64 {
        self.as_logical_slice().get_ts()
    }
}

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BasicPhysicalKey(pub Vec<u8>);

impl Debug for BasicPhysicalKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use hex::ToHex;
        self.0.as_slice().write_hex_upper(f)
    }
}

impl Display for BasicPhysicalKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl KeyLike for BasicPhysicalKey {}

impl BufferWriter for BasicPhysicalKey {
    #[inline(never)]
    unsafe fn bytes_mut(&mut self, size: usize) -> &mut [u8] {
        self.0.bytes_mut(size)
    }

    #[inline(never)]
    unsafe fn advance_mut(&mut self, count: usize) {
        self.0.advance_mut(count)
    }

    #[inline(never)]
    fn write_bytes(&mut self, values: &[u8]) -> codec::Result<()> {
        self.0.write_bytes(values)
    }
}

impl Borrow<BasicPhysicalKeySlice> for BasicPhysicalKey {
    fn borrow(&self) -> &BasicPhysicalKeySlice {
        self.as_physical_slice()
    }
}

impl Deref for BasicPhysicalKey {
    type Target = BasicPhysicalKeySlice;

    fn deref(&self) -> &Self::Target {
        self.as_physical_slice()
    }
}

impl PhysicalKey for BasicPhysicalKey {
    const PHYSICAL_PREFIX: &'static [u8] = b"";
    type Slice = BasicPhysicalKeySlice;

    #[inline(never)]
    fn _new_from_vec(vec: Vec<u8>) -> Self {
        BasicPhysicalKey(vec)
    }

    #[inline(never)]
    fn _vec_ref(&self) -> &Vec<u8> {
        &self.0
    }

    #[inline(never)]
    fn _vec_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }

    #[inline(never)]
    fn _into_vec(self) -> Vec<u8> {
        self.0
    }
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BasicPhysicalKeySlice(pub [u8]);

impl BasicPhysicalKeySlice {
    #[inline(never)]
    pub fn from_logical_slice(s: &LogicalKeySlice) -> &Self {
        // For `BasicPhysicalKeySlice`, its logical slice is equal to the physical slice, so
        // we can do the transform directly.
        Self::from_physical_std_slice(s.as_std_slice())
    }
}

impl Debug for BasicPhysicalKeySlice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use hex::ToHex;
        self.as_physical_std_slice().write_hex_upper(f)
    }
}

impl Display for BasicPhysicalKeySlice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl KeyLike for BasicPhysicalKeySlice {}

impl PhysicalKeySlice for BasicPhysicalKeySlice {
    type OwnedKey = BasicPhysicalKey;

    // TODO: Only to support `impl Key for ToPhysicalKeySlice<T>`. To be removed.
    type LegacyKeySliceOwner = ();

    // TODO: Only to support `impl Key for ToPhysicalKeySlice<T>`. To be removed.
    #[inline(never)]
    fn from_legacy_key(key: &Key) -> PKContainer<'_, (), Self> {
        let pk_slice = BasicPhysicalKeySlice::from_logical_slice(key.as_logical_key_slice());
        pk_slice.to_physical_slice_container()
    }

    #[inline(never)]
    fn as_physical_std_slice(&self) -> &[u8] {
        &self.0
    }

    #[inline(never)]
    fn from_physical_std_slice(s: &[u8]) -> &Self {
        unsafe { &*(s as *const [u8] as *const Self) }
    }

    #[inline(never)]
    fn as_logical_slice(&self) -> &LogicalKeySlice {
        LogicalKeySlice::from_std_slice(&self.0)
    }
}

// Owned Logical Key is intentionally not provided to avoid abuse.

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogicalKeySlice(pub [u8]);

impl Debug for LogicalKeySlice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use hex::ToHex;
        self.as_std_slice().write_hex_upper(f)
    }
}

impl Display for LogicalKeySlice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl KeyLike for LogicalKeySlice {}

impl LogicalKeySlice {
    #[inline(never)]
    pub fn from_std_slice(s: &[u8]) -> &Self {
        unsafe { &*(s as *const [u8] as *const Self) }
    }

    // FIXME: This is for compatibility. To be removed.
    #[inline(never)]
    pub fn from_legacy_key(k: &Key) -> &Self {
        Self::from_std_slice(k.as_encoded().as_slice())
    }

    #[inline(never)]
    pub fn as_std_slice(&self) -> &[u8] {
        &self.0
    }

    // FIXME: This is a MVCC knowledge.
    #[inline(never)]
    pub fn as_physical_slice_without_ts(&self) -> &LogicalKeySlice {
        let s = self.as_std_slice();
        Self::from_std_slice(&s[..s.len() - 8])
    }

    // FIXME: This is a MVCC knowledge.
    #[inline(never)]
    pub fn alloc_to_user_vec(&self) -> codec::Result<Vec<u8>> {
        self.as_std_slice().read_comparable_bytes()
    }

    #[inline(never)]
    pub fn get_ts(&self) -> u64 {
        let s = self.as_std_slice();
        let mut s_ts = &s[s.len() - 8..];
        s_ts.read_u64_desc().unwrap()
    }
}

impl Deref for LogicalKeySlice {
    type Target = [u8];

    #[inline(never)]
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl DerefMut for LogicalKeySlice {
    #[inline(never)]
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

// FIXME: This is for compatibility. To be removed.
pub trait ToPhysicalKeySlice<Target: PhysicalKeySlice + ?Sized> {
    type SliceOwner;

    // Called `to_xxx` instead of `as_xxx` to explicitly allow extra cost when
    // doing the conversion.
    fn to_physical_slice_container(&self) -> PKContainer<'_, Self::SliceOwner, Target>;
}

// FIXME: This is for compatibility. To be removed.
/// A helper structure which groups a value and a reference together,
/// optionally with a life time. It's some how similar to OwningRef,
/// but allow the reference to be not depend on the value (but guarded
/// by an extra life time)
pub struct PKContainer<'a, PKOwner, PKSlice: PhysicalKeySlice + ?Sized> {
    _phantom: PhantomData<&'a PKSlice>,
    _owner: PKOwner,
    reference: *const PKSlice,
}

impl<'a, PKOwner, PKSlice: PhysicalKeySlice + ?Sized> PKContainer<'a, PKOwner, PKSlice> {
    pub unsafe fn new(owner: PKOwner, reference: *const PKSlice) -> Self {
        Self {
            _phantom: PhantomData,
            _owner: owner,
            reference,
        }
    }
}

impl<PKOwner, PKSlice: PhysicalKeySlice + ?Sized> Deref for PKContainer<'_, PKOwner, PKSlice> {
    type Target = PKSlice;

    fn deref(&self) -> &PKSlice {
        unsafe { &*self.reference }
    }
}

impl<T: ?Sized, U: ?Sized> ToPhysicalKeySlice<U> for &T
where
    U: PhysicalKeySlice,
    T: ToPhysicalKeySlice<U>,
{
    type SliceOwner = T::SliceOwner;

    #[inline(never)]
    fn to_physical_slice_container(&self) -> PKContainer<'_, T::SliceOwner, U> {
        <T as ToPhysicalKeySlice<U>>::to_physical_slice_container(*self)
    }
}

impl<T: ?Sized, U: ?Sized> ToPhysicalKeySlice<U> for &mut T
where
    U: PhysicalKeySlice,
    T: ToPhysicalKeySlice<U>,
{
    type SliceOwner = T::SliceOwner;

    #[inline(never)]
    fn to_physical_slice_container(&self) -> PKContainer<'_, T::SliceOwner, U> {
        <T as ToPhysicalKeySlice<U>>::to_physical_slice_container(*self)
    }
}

impl ToPhysicalKeySlice<BasicPhysicalKeySlice> for BasicPhysicalKeySlice {
    // Any PhysicalKeySlice itself implements ToPhysicalKeySlice.
    type SliceOwner = ();

    #[inline(never)]
    fn to_physical_slice_container(&self) -> PKContainer<'_, (), BasicPhysicalKeySlice> {
        let r = self as *const BasicPhysicalKeySlice;
        unsafe { PKContainer::new((), r) }
    }
}

impl ToPhysicalKeySlice<BasicPhysicalKeySlice> for BasicPhysicalKey {
    // Any PhysicalKey convert to its slice is zero cost and does not need to carry an
    // extra owned value.
    type SliceOwner = ();

    #[inline(never)]
    fn to_physical_slice_container(&self) -> PKContainer<'_, (), BasicPhysicalKeySlice> {
        self.as_physical_slice().to_physical_slice_container()
    }
}

impl<T: PhysicalKeySlice + ?Sized> ToPhysicalKeySlice<T> for Key {
    type SliceOwner = T::LegacyKeySliceOwner;

    #[inline(never)]
    fn to_physical_slice_container(&self) -> PKContainer<'_, Self::SliceOwner, T> {
        T::from_legacy_key(self)
    }
}

impl ToPhysicalKeySlice<BasicPhysicalKeySlice> for [u8] {
    // Allows `&[u8]` to be used directly as a `BasicPhysicalKeySlice`.
    type SliceOwner = ();

    #[inline(never)]
    fn to_physical_slice_container(&self) -> PKContainer<'_, (), BasicPhysicalKeySlice> {
        let pk_slice = BasicPhysicalKeySlice::from_physical_std_slice(self);
        pk_slice.to_physical_slice_container()
    }
}

impl<const N: usize> ToPhysicalKeySlice<BasicPhysicalKeySlice> for [u8; N] {
    type SliceOwner = ();

    #[inline(never)]
    fn to_physical_slice_container(&self) -> PKContainer<'_, (), BasicPhysicalKeySlice> {
        (&self[..]).to_physical_slice_container()
    }
}

impl ToPhysicalKeySlice<BasicPhysicalKeySlice> for Vec<u8> {
    type SliceOwner = ();

    #[inline(never)]
    fn to_physical_slice_container(&self) -> PKContainer<'_, (), BasicPhysicalKeySlice> {
        self.as_slice().to_physical_slice_container()
    }
}

// Assert PKContainer<'a, (), _> has zero space cost.
assert_eq_size!(
    PKContainer<'static, (), BasicPhysicalKeySlice>,
    &'static BasicPhysicalKeySlice
);
