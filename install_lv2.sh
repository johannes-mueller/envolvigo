#!/bin/bash

cargo build --release || exit 1

[ ! -d $HOME/.lv2 ] && mkdir $HOME/.lv2
[ ! -d $HOME/.lv2/envolvigo.lv2 ] && mkdir $HOME/.lv2/envolvigo.lv2

cp target/release/libenvolvigo_lv2.so $HOME/.lv2/envolvigo.lv2/ || exit
cp target/release/libenvolvigo_lv2_ui.so $HOME/.lv2/envolvigo.lv2/ || exit
cp lv2/*ttl $HOME/.lv2/envolvigo.lv2/ || exit

echo
echo envolvigo.lv2 successfully installed
