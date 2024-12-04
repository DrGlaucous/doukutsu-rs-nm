#real programmers will vomit at this makefile (I did not vomit)


all:
	ios system



#requires proper targets and lipo to be installed
ios: release-path = ./target/aarch64-apple-ios/release
ios:
	cargo lipo --release
	mv ${release-path}/libdoukutsu-rs-libretro.dylib ${release-path}/doukutsu_rs_libretro.dylib
	codesign -s - ${release-path}/doukutsu_rs_libretro.dylib
	codesign -d -v ${release-path}/doukutsu_rs_libretro.dylib


#ditto, but for android ndk
android:
	cargo-ndk build --release

#builds it for whatever system we're on right now
system:
	cargo build --release


clean:
	cargo clean








