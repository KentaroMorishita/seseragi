# Native BigInt arithmetic across an opaque package boundary

The vendor owns Key and its Add/Mul/Pow/Zero/One/Show instances. The caller
keeps saved generic stages and the imported standard division-error alias.
Original raw TypeScript in both profiles is checked with strict and
skipLibCheck:false and run in fresh Bun/Node processes against native.
