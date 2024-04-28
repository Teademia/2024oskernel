IMG_NAME =fs.img
IMG_SIZE_MB = 100

run: rootfs
	@cd os && make run
rootfs: img
	@mcopy -i fs.img -s user_apps/* ::

img:
	@dd if=/dev/zero of=$(IMG_NAME) bs=1M count=$(IMG_SIZE_MB)
	@mkfs.fat -F 32 $(IMG_NAME)

clean:
	@rm fs.img -f
	@cd os && rm target -rf
	@cd fat32_fs && rm target -rf
	@cd 
	@rm user_apps/* -rf
	@rm riscv-syscalls-testing/user/build -rf

server:
	@cd os && make server

client:
	@cd os && make client

test_case:
	@rm riscv-syscalls-testing/user/build -rf
	@cd riscv-syscalls-testing/user && bash build-oscomp.sh && cp build/riscv64/* ../../user_apps