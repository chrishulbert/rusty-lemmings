.PHONY: help macos

help:
	cat Makefile

macos:
	rm -rf Rusty\ Lemmings.app
	rm -f Rusty\ Lemmings.app.zip
	MACOSX_DEPLOYMENT_TARGET=11.0 cargo build --release
	mkdir Rusty\ Lemmings.app
	mkdir Rusty\ Lemmings.app/Contents
	mkdir Rusty\ Lemmings.app/Contents/Resources
	mkdir Rusty\ Lemmings.app/Contents/MacOS
	echo APPL???? > Rusty\ Lemmings.app/Contents/PkgInfo
	cp icon/Assets.car Rusty\ Lemmings.app/Contents/Resources
	cp target/release/rusty-lemmings Rusty\ Lemmings.app/Contents/MacOS
	cp macos/Info.plist Rusty\ Lemmings.app/Contents
	codesign -s RustyLemmings Rusty\ Lemmings.app # https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/Procedures/Procedures.html
	zip -r Rusty\ Lemmings.app.zip Rusty\ Lemmings.app
	@echo "!!! Ensure the Cargo.toml bevy dependency isn't dynamic !!!"
	@echo "!!! Ensure egui is disabled !!!"

macos-intel:
	cargo build --release --target=x86_64-apple-darwin
