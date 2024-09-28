name := 'oxyclock'

rootdir := ''

appdir := rootdir / 'usr' / 'share' / 'applications'

bin-src := 'target' / 'release' / name
bin-dest := rootdir / 'usr' / 'bin'/ name

entry := 'oxyclock.desktop'

build:
  cargo build --release

install:
  install -Dm644 'resources/{{entry}} '{{appdir}}/{{entry}}'
  install -Dm755 bin-src bin-dest
