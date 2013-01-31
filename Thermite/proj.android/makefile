all:
	./build_native.sh
	ant -Dsdk.dir=/home/delorenj/sdk debug
run:
	adb -d uninstall tractorsandham.labs.thermite
	adb -d install bin/Thermite-debug.apk
	adb -d logcat | grep cocos2d
clean:
	rm -rf libs/
	rm -rf obj/
	rm -rf bin/*
	rm -rf gen/
	rm -rf assets/*
