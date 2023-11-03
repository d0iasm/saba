#!/bin/bash -xe

HOME_PATH=$PWD
TARGET_PATH=$PWD"/build"
OS_PATH=$TARGET_PATH"/wasabi"
APP_PATH=$OS_PATH"/app/toybr"

# execute `mkdir build/` if it doesn't exist
if [ -d $TARGET_PATH ]
then
  echo $TARGET_PATH" exists"
else
  echo $TARGET_PATH" doesn't exist"
  mkdir $TARGET_PATH
fi

# install Wasabi OS (https://github.com/hikalium/wasabi)
if [ -d $OS_PATH ]
then
  echo $OS_PATH" exists"
  echo "pulling new changes..."
  cd $OS_PATH
  git pull
else
  echo $OS_PATH" doesn't exist"
  echo "cloning wasabi project..."
  cd $TARGET_PATH
  git clone git@github.com:hikalium/wasabi.git
fi

# go back to the application top directory
cd $HOME_PATH

# create app/toybr in Wasabi OS if it doesn't exist
if [ -d $APP_PATH ]
then
  echo $APP_PATH" exists"
else
  echo $APP_PATH" doesn't exist"
  mkdir $APP_PATH
fi

# copy Toybr application except `target`, `.git` and `build` directories to Wasabi
echo "copying the toybr application to wasabi OS..."
cp -R `ls -A ./ | grep -v "target" | grep -v ".git" | grep -v "build"` $APP_PATH

cd $OS_PATH
make

# add app target to Wasabi OS
mv Cargo.toml Cargo.toml.original
sed 's/members = \[/members = \[\n    "app\/toybr",/' Cargo.toml.original > Cargo.toml

make run

cd $HOME_PATH
