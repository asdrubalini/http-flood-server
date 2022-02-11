PROJECT=http-flood-server

build:
	cargo build --target x86_64-unknown-linux-musl --release
	strip target/x86_64-unknown-linux-musl/release/$(PROJECT)
	cp target/x86_64-unknown-linux-musl/release/$(PROJECT) .

install:
	[ -d /usr/bin/$(PROJECT)/ ] || mkdir /usr/bin/$(PROJECT)/
	cp ./$(PROJECT) /usr/bin/$(PROJECT)/$(PROJECT)
	cp ./$(PROJECT).service /etc/systemd/system/$(PROJECT).service

uninstall:
	rm -r /usr/bin/$(PROJECT)/
	rm /etc/systemd/system/$(PROJECT).service

