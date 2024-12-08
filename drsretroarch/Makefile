#real programmers will vomit at this makefile (I did not vomit)
#this makefile is not strictly required, but does make building more convenient


#builds it for whatever system we're on right now 
system: release-path = ./target/release
system:
	cargo build --release
	make rename-files release-path=${release-path}

#trims leading "lib" off built files. Must be done in seperate command to make sure none of it executes before "system" is finished
rename-files:
	$(eval FILES := $(wildcard $(release-path)/lib*))

	@for file in ${FILES}; do \
		if [ -e "$$file" ]; then \
			newname=$$(basename "$$file" | sed 's/^lib\(.*\)$/\1_libretro'); \
			mv "$$file" "$(release-path)/$$newname"; \
			echo "Renamed: $$file -> $(release-path)/$$newname"; \
		fi; \
	done



all-apple:
	make ios system

all-linux:
	make android system

#requires cargo-lipo to be installed (the only target we're looking at here is the aarch64 one, which is current physical apple deivces)
ios: release-path = ./target/aarch64-apple-ios/release
ios:
	rustup target add aarch64-apple-ios
	cargo lipo --release
	mv ${release-path}/libdoukutsu_rs.dylib ${release-path}/doukutsu_rs_libretro.dylib
	codesign -s - ${release-path}/doukutsu_rs_libretro.dylib
	codesign -d -v ${release-path}/doukutsu_rs_libretro.dylib


#ditto, but for android ndk
android:
	rustup target add aarch64-linux-android
	rustup target add armv7-linux-androideabi
	cargo ndk build --release
	make rename-files release-path=./target/aarch64-linux-android/release
	make rename-files release-path=./target/armv7-linux-androideabi/release


clean:
	cargo clean


