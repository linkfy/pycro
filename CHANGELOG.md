# Changelog

## [0.5.0](https://github.com/linkfy/pycro/compare/pycro_cli-v0.4.4...pycro_cli-v0.5.0) (2026-03-18)


### Features

* **governance:** close phase16 workflow hardening ([73d9b9b](https://github.com/linkfy/pycro/commit/73d9b9b0cd51336b4afac0d4dfcddd50544f5673))
* **web:** close phase 17 wasm poc build and runtime stability ([a3c8abd](https://github.com/linkfy/pycro/commit/a3c8abd86e348da88cef863080ca51229376ed92))

## [0.4.4](https://github.com/linkfy/pycro/compare/pycro_cli-v0.4.3...pycro_cli-v0.4.4) (2026-03-17)


### Bug Fixes

* **ci:** pin macos deployment target for artifact workflows ([f3cf77b](https://github.com/linkfy/pycro/commit/f3cf77b193dba632494d3870743e89ee7284eca3))
* **ci:** use static crt for windows artifacts ([13c1b76](https://github.com/linkfy/pycro/commit/13c1b763aed22b8efc936927ce9eb93208232119))
* **input:** map letter keys and drop ineffective polling fallback ([82f6ed4](https://github.com/linkfy/pycro/commit/82f6ed4959ee08e985d624a0aceaf002dc43d0b3))
* **windows-input:** patch miniquad rawinput target and add diagnostic scenario ([d3893db](https://github.com/linkfy/pycro/commit/d3893db6a49a75da14544cf635d828eb3c80da39))

## [0.4.3](https://github.com/linkfy/pycro/compare/pycro_cli-v0.4.2...pycro_cli-v0.4.3) (2026-03-16)


### Bug Fixes

* **input:** add pressed-state fallback for windows key polling ([0f12a3e](https://github.com/linkfy/pycro/commit/0f12a3ec89e371b8420a2b0e7824d2005da32c31))

## [0.4.2](https://github.com/linkfy/pycro/compare/pycro_cli-v0.4.1...pycro_cli-v0.4.2) (2026-03-16)


### Bug Fixes

* **runtime:** register frozen stdlib during vm init ([17edfa2](https://github.com/linkfy/pycro/commit/17edfa2a602e3bbe2b2363935254609763ddc45b))

## [0.4.1](https://github.com/linkfy/pycro/compare/pycro_cli-v0.4.0...pycro_cli-v0.4.1) (2026-03-16)


### Bug Fixes

* **runtime:** freeze stdlib and clarify init-first quickstart ([8bb64ab](https://github.com/linkfy/pycro/commit/8bb64abfce06c59774f177dff9fee101cc2103f2))

## [0.4.0](https://github.com/linkfy/pycro/compare/pycro_cli-v0.3.0...pycro_cli-v0.4.0) (2026-03-16)


### Features

* **ci:** adopt develop-first flow with per-push test artifacts ([e657327](https://github.com/linkfy/pycro/commit/e657327db2bdbc09fa80620cf8e15575f3596034))
* **phase12:** close vec2/color coercion and key enum contract ([0cb6233](https://github.com/linkfy/pycro/commit/0cb6233d89953f87c1410028359a7a78afd8439a))
* **project-build:** close phase 15 desktop embedded payload ([e3c9ef7](https://github.com/linkfy/pycro/commit/e3c9ef71acfd767e42aafe524186b142e99872d8))
* **project:** close phase 14 build foundation and CLI contract ([961c4cd](https://github.com/linkfy/pycro/commit/961c4cdb40fdfca590d4b7400e7b4e6b6a78cdb4))


### Bug Fixes

* **cli:** default generate_stubs to project-local pycro.pyi ([a9106f2](https://github.com/linkfy/pycro/commit/a9106f2164bf4f67dfe0b373a4271f28ee268f0c))

## [0.3.0](https://github.com/linkfy/pycro/compare/pycro_cli-v0.2.0...pycro_cli-v0.3.0) (2026-03-13)


### Features

* **runtime:** enforce update-only lifecycle and project bootstrap defaults ([9c4e43a](https://github.com/linkfy/pycro/commit/9c4e43a57760f932482d126924906b65a7d253cd))

## [0.2.0](https://github.com/linkfy/pycro/compare/pycro_cli-v0.1.2...pycro_cli-v0.2.0) (2026-03-13)


### Features

* **cli:** add pycro init project scaffold command and activate phase09 ([4e87bcd](https://github.com/linkfy/pycro/commit/4e87bcdb86c728af6fb1052829450e20109e281e))


### Bug Fixes

* **docs:** clarify releasable commit types for release-please ([e3a8624](https://github.com/linkfy/pycro/commit/e3a8624550807281765675f7864850151746671c))

## [0.1.2](https://github.com/linkfy/pycro/compare/pycro_cli-v0.1.1...pycro_cli-v0.1.2) (2026-03-13)


### Bug Fixes

* **ci:** use macos-latest runner labels for release artifacts ([0ede211](https://github.com/linkfy/pycro/commit/0ede21191a30c98a667e8530c4f4e5172e1d6b05))

## [0.1.1](https://github.com/linkfy/pycro/compare/pycro_cli-v0.1.0...pycro_cli-v0.1.1) (2026-03-13)


### Bug Fixes

* **ci:** harden release-please parsing and PR token policy ([a995628](https://github.com/linkfy/pycro/commit/a995628a69c76b9fbeabc9f81724ec7022ab1d36))
* **ci:** use valid release-please-action v4 tag ([9018889](https://github.com/linkfy/pycro/commit/901888927a11c95b36a4be3183fc68badde7e3a2))
