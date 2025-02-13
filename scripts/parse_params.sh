#!/bin/sh

usage="Usage: fam [ all | test | fasttest | coverage ] [options]";

for var in "$@"; do
	case "$var" in
	-s)
		s=1
		;;
	--mrustc)
		usemrustc=1
		;;
	--filter=*)
		filter=${var#*=}
		;;
	all)
		all=1;
		ccflags=-O3
		;;
	--debug)
		debug="-C debuginfo=2"
		;;
	--output=*)
		output=${var#*=}
		;;
	--update-docs)
		updatedocs=--update-docs
		;;
	fasttest)
		fasttest=1;
		ccflags=-O3
		rustflags="-C opt-level=3"
		;;
	--with-cc=*)
                cc=${var#*=}
                ;;
	--with-mrustc=*)
		mrustc=${var#*=}
		;;
	coverage)
		coverage=1;
		rustflags="-C instrument-coverage -C opt-level=0"
		;;
	test)
		test=1;
		;;
	clean)
		clean=1;
		;;
	*)
		echo "Unrecognized option: '$var'"
		echo $usage;
		exit 1;
	esac
done

if [ "$test" != "1" ]  && [ "$coverage" != "1" ] && [ "$fasttest" != "1" ] && [ "$clean" != "1" ]; then
	all=1;
fi

