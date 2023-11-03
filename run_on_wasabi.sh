#!/bin/bash -xe

HOME_PATH=$PWD
BASE_PATH="../"
OS_PATH=$BASE_PATH"wasabi"
APP_PATH=$OS_PATH"/app/toybr"

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
  cd $BASE_PATH
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

# copy Toybr application except `os` and `target` directories to Wasabi
echo "copying the toybr application to wasabi OS..."
cp -R `ls -A ./ | grep -v "target" | grep -v ".git"` $APP_PATH

cd $OS_PATH
make font
make run

cd $HOME_PATH
