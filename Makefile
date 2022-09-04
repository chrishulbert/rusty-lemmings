.PHONY: help macos

help:
	cat Makefile

macos:
	rm -rf Rusty\ Lemmings.app
	cargo build --release
	mkdir Rusty\ Lemmings.app
	mkdir Rusty\ Lemmings.app/Contents
	mkdir Rusty\ Lemmings.app/Contents/Resources
	mkdir Rusty\ Lemmings.app/Contents/MacOS
	echo APPL???? > Rusty\ Lemmings.app/Contents/PkgInfo
	cp icon/Assets.car Rusty\ Lemmings.app/Contents/Resources
	cp target/release/rusty-lemmings Rusty\ Lemmings.app/Contents/MacOS
	cp macos/Info.plist Rusty\ Lemmings.app/Contents
	@echo "!!! Ensure the Cargo.toml bevy dependency isn't dynamic !!!"
	@echo "!!! Ensure egui is disabled !!!"
