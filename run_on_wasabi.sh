#!/bin/bash -xe

HOME_PATH=$PWD
TARGET_PATH=$PWD"/build"
OS_PATH=$TARGET_PATH"/wasabi"
APP_NAME="saba"
MAKEFILE_PATH=$HOME_PATH"/Makefile"

# execute `mkdir build/` if it doesn't exist
if [ -d $TARGET_PATH ]
then
  echo $TARGET_PATH" exists"
else
  echo $TARGET_PATH" doesn't exist"
  mkdir $TARGET_PATH
fi

# install Wasabi OS (https://github.com/hikalium/wasabi)
# You should manually remove wasabi/ if it's conflict via `rm -rf $OS_PATH`
if [ -d $OS_PATH ]
then
  echo $OS_PATH" exists"
  echo "pulling new changes..."
  cd $OS_PATH
  git pull origin for_saba
else
  echo $OS_PATH" doesn't exist"
  echo "cloning wasabi project..."
  cd $TARGET_PATH
  git clone --branch for_saba git@github.com:hikalium/wasabi.git
fi

# go back to the application top directory
cd $HOME_PATH

# download Makefile if it doesn't exist
if [ ! -f $MAKEFILE_PATH ]; then
  echo "downloading Makefile..."
  wget https://raw.githubusercontent.com/hikalium/wasabi/for_saba/external_app_template/Makefile
fi

make build
$OS_PATH/scripts/run_with_app.sh ./target/x86_64-unknown-none/release/$APP_NAME
