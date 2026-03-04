# SPDX-License-Identifier: GPL-2.0

kernel_version := `uname -r`
kdir := env_var_or_default('KDIR', f'/lib/modules/{{kernel_version}}/build')
pwd := justfile_directory()

default: build

build:
  make -C {{kdir}} M={{pwd}}

modules_install: build
	make -C {{kdir}} M={{pwd}} modules_install

clean:
	make -C {{kdir}} M={{pwd}} clean

rust-analyzer:
  make -C {{kdir}} M={{pwd}} rust-analyzer

load:
    sudo insmod bunny.ko

unload:
    sudo rmmod bunny

test:
    cat /dev/bunny
