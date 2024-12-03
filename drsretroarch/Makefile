#real programmers will vomit at this makefile (I did not vomit)


all:
	ios system



#requires proper targets and lipo to be installed
ios: release-path = ./target/aarch64-apple-ios/release
ios:
	cargo lipo --release
	mv ${release-path}/libdrsretroarch.dylib ${release-path}/drsretroarch.dylib
	codesign -s - ${release-path}/drsretroarch.dylib
	codesign -d -v ${release-path}/drsretroarch.dylib


#ditto, but for android ndk
android:
	cargo-ndk build --release

#builds it for whatever system we're on right now
system:
	cargo build --release


clean:
	cargo clean








