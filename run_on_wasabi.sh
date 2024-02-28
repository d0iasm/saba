#!/bin/bash -xe

HOME_PATH=$PWD
TARGET_PATH=$PWD"/build"
OS_PATH=$TARGET_PATH"/wasabi"
APP_PATH=$OS_PATH"/app/saba"

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
  git pull
else
  echo $OS_PATH" doesn't exist"
  echo "cloning wasabi project..."
  cd $TARGET_PATH
  git clone git@github.com:hikalium/wasabi.git
fi

# go back to the application top directory
cd $HOME_PATH

# create app/saba in Wasabi OS if it doesn't exist
if [ -d $APP_PATH ]
then
  echo $APP_PATH" exists"
else
  echo $APP_PATH" doesn't exist"
  mkdir $APP_PATH
fi

# copy Toybr application except `target`, `.git` and `build` directories to Wasabi
echo "copying the saba application to wasabi OS..."
cp -R `ls -A ./ | grep -v "target" | grep -v ".git" | grep -v "build"` $APP_PATH

cd $OS_PATH

# update Cargo.toml to add saba
# this is very hacky and not stable
mv Cargo.toml Cargo.toml.original
if grep -Fq "app/saba" Cargo.toml.original
then
  echo "app/saba already exists in Cargo.toml"
  mv Cargo.toml.original Cargo.toml
else
  sed 's/members = \[/members = \[\n    "app\/saba",/' Cargo.toml.original >| Cargo.toml
fi

# update Makefile to add saba
# this is very hacky and not stable
mv Makefile Makefile.original
if grep -Fq "app/saba" Makefile.original
then
  echo "app/saba already exists in Makefile"
  mv Makefile.original Makefile
else
  sed 's/make -C app\/window0/make -C app\/window0\n\tmake -C app\/saba/' Makefile.original >| Makefile
fi

make
make run

cd $HOME_PATH
