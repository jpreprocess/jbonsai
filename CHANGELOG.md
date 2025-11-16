# Changelog

## [0.4.1](https://github.com/jpreprocess/jbonsai/compare/v0.4.0...v0.4.1) (2025-11-16)


### Performance Improvements

* unify loops in mlsafir ([#112](https://github.com/jpreprocess/jbonsai/issues/112)) ([46278ad](https://github.com/jpreprocess/jbonsai/commit/46278ad7b18a5c2f0f3f0fc5efe632fe761932d0))
* use fmla.2d ([#114](https://github.com/jpreprocess/jbonsai/issues/114)) ([55b2214](https://github.com/jpreprocess/jbonsai/commit/55b2214089ba301b898f0b3fd822740f87dc6358))

## [0.4.0](https://github.com/jpreprocess/jbonsai/compare/v0.3.0...v0.4.0) (2025-10-18)


### ⚠ BREAKING CHANGES

* Remove unstable SIMD optimization

### Features

* Remove unstable SIMD optimization ([4392df6](https://github.com/jpreprocess/jbonsai/commit/4392df662bbd12710af79bd9f1269a983e96a682))


### Bug Fixes

* warnings ([#97](https://github.com/jpreprocess/jbonsai/issues/97)) ([506c136](https://github.com/jpreprocess/jbonsai/commit/506c136ba69c0c824aef85814b21cd4a4ecfeeb8))


### Performance Improvements

* Optimize excitation ([#108](https://github.com/jpreprocess/jbonsai/issues/108)) ([be245e1](https://github.com/jpreprocess/jbonsai/commit/be245e10941e26bd3f895077953eca08dc8aa626))


### Dependencies

* update rust crate thiserror to v2.0.14 ([#102](https://github.com/jpreprocess/jbonsai/issues/102)) ([6403653](https://github.com/jpreprocess/jbonsai/commit/6403653a6db054538d67786ac68e423a2db48e3e))
* update rust crate thiserror to v2.0.15 ([#103](https://github.com/jpreprocess/jbonsai/issues/103)) ([77ecdcc](https://github.com/jpreprocess/jbonsai/commit/77ecdccb424bb1fe2c9f6fa5803f82b3b16e6a3a))
* update rust crate thiserror to v2.0.16 ([#104](https://github.com/jpreprocess/jbonsai/issues/104)) ([fe69bd3](https://github.com/jpreprocess/jbonsai/commit/fe69bd30c3ebf335145376386b3208b731bc3f23))

## [0.3.0](https://github.com/jpreprocess/jbonsai/compare/v0.2.2...v0.3.0) (2025-05-10)


### ⚠ BREAKING CHANGES

* move `jbonsai::model::load_htsvoice_file` to `jbonsai::model::load_htsvoice_from_bytes`

### Features

* add Engine::load_from_bytes() ([#59](https://github.com/jpreprocess/jbonsai/issues/59)) ([13cf2f3](https://github.com/jpreprocess/jbonsai/commit/13cf2f3fd77faf341e973ca87061b8ebdca5baa7))
* edition 2024 ([#77](https://github.com/jpreprocess/jbonsai/issues/77)) ([1f3d741](https://github.com/jpreprocess/jbonsai/commit/1f3d7417c7cc4b8cd98fa578744dc26c41b19365))


### Bug Fixes

* clippy ([#89](https://github.com/jpreprocess/jbonsai/issues/89)) ([4c23191](https://github.com/jpreprocess/jbonsai/commit/4c231915c26ab0aa6859e535c14b05494f78f010))
* **deps:** update dependencies (non-major) ([#74](https://github.com/jpreprocess/jbonsai/issues/74)) ([f4d6d0b](https://github.com/jpreprocess/jbonsai/commit/f4d6d0b1efa4b793a0f1e8e7eaa2372fcbd6c64e))

## [0.2.2](https://github.com/jpreprocess/jbonsai/compare/v0.2.1...v0.2.2) (2025-02-08)


### Continuous Integration

* inherit secrets in release workflow ([#71](https://github.com/jpreprocess/jbonsai/issues/71)) ([5b27727](https://github.com/jpreprocess/jbonsai/commit/5b27727a03f60f3b686ed24b847a316b6529b02b))

## [0.2.1](https://github.com/jpreprocess/jbonsai/compare/v0.2.0...v0.2.1) (2025-02-08)


### Continuous Integration

* fix publish command ([#69](https://github.com/jpreprocess/jbonsai/issues/69)) ([d042103](https://github.com/jpreprocess/jbonsai/commit/d0421035db49cb8d732fb4c00ae2dfccec07462d))

## [0.2.0](https://github.com/jpreprocess/jbonsai/compare/v0.1.1...v0.2.0) (2025-02-03)


### ⚠ BREAKING CHANGES

* pub(crate) for ::model::tests ([#55](https://github.com/jpreprocess/jbonsai/issues/55))

### Features

* add SIMD impl for MLSA::fir ([#61](https://github.com/jpreprocess/jbonsai/issues/61)) ([09c31cd](https://github.com/jpreprocess/jbonsai/commit/09c31cdb9f2d926201105adc9303695b91dea1ce))


### Dependencies

* nom v8.0 ([#65](https://github.com/jpreprocess/jbonsai/issues/65)) ([34fdb09](https://github.com/jpreprocess/jbonsai/commit/34fdb092acc5c375338b1759644dfd6f77aaac8e))
* update 20250202 ([#66](https://github.com/jpreprocess/jbonsai/issues/66)) ([c3a0fd2](https://github.com/jpreprocess/jbonsai/commit/c3a0fd24482f640abb4d5b42b4f71bb6ce860602))


### Miscellaneous Chores

* pub(crate) for ::model::tests ([#55](https://github.com/jpreprocess/jbonsai/issues/55)) ([f19159b](https://github.com/jpreprocess/jbonsai/commit/f19159bf0a8214e7d5967ed7d19ee0baa190d64c))

## [0.1.1](https://github.com/jpreprocess/jbonsai/compare/v0.1.0...v0.1.1) (2024-05-26)


### Bug Fixes

* Fix panic when empty string is provided and speed is set ([#50](https://github.com/jpreprocess/jbonsai/issues/50)) ([f67f2a2](https://github.com/jpreprocess/jbonsai/commit/f67f2a2473e77dcd4f4705051eba041ef7abe186))
