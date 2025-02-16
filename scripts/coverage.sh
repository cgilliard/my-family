#!/bin/sh

update_docs=0;
for var in "$@"
do
case "$var" in --update-docs)
        update_docs=1;
        ;;
esac
done

cur_file='';
line_count=0;
cov_count=0;
line_count_sum=0;
cov_count_sum=0;

if [ "$NO_COLOR" = "" ]; then
   GREEN="\033[32m";
   CYAN="\033[36m";
   YELLOW="\033[33m";
   BRIGHT_RED="\033[91m";
   RESET="\033[0m";
   BLUE="\033[34m";
else
   GREEN="";
   CYAN="";
   YELLOW="";
   BRIGHT_RED="";
   RESET="";
   BLUE="";
fi

echo "Code Coverage Report for commit: $GREEN`git log -1 | grep "^commit " | cut -f2 -d ' '`$RESET";
echo "$BLUE----------------------------------------------------------------------------------------------------$RESET";

for line in $(cat /tmp/coverage.txt)
do
	if [ "`echo $line | grep "^SF:"`" != "" ]; then
		cur_file=`echo $line | cut -f2 -d ':'`;
	fi
	if [ "`echo $line | grep "^DA:"`" != "" ]; then
		#echo "da: $line";
		line_count=$((1 + line_count));
		line_count_sum=$((1 + line_count_sum));
		if [ "`echo $line | cut -f2 -d ','`" != "0" ]; then
			cov_count=$((1 + cov_count));
			cov_count_sum=$((1 + cov_count_sum));
		fi
	fi
	if [ "$line" = "end_of_record" ]; then
		percent=100;
		if [ "$line_count" != "0" ]; then
			percent=$(((cov_count * 100) / line_count));
		fi
		line_fmt="($cov_count/$line_count)";
		printf "Cov: $GREEN%3s%%$RESET Lines: $YELLOW%10s$RESET File: $CYAN%s$RESET\n" "$percent" "$line_fmt" "$cur_file"
		line_count=0;
		cov_count=0;
	fi
done
echo "$BLUE----------------------------------------------------------------------------------------------------$RESET";

percent=100;
if [ "$line_count_sum" != "0" ]; then
	percent=$(((cov_count_sum * 100) / line_count_sum));
fi
echo "Summary: $GREEN$percent%$RESET Lines: $YELLOW($cov_count_sum/$line_count_sum)$RESET!"
codecov=`printf "%.2f" $percent`;
timestamp=`date +%s`

if [ $update_docs = 1 ]; then
	echo "$codecov" > /tmp/cc_final;
	echo "$timestamp $codecov $cov_count_sum $line_count_sum" >> ./docs/cc.txt
	./scripts/update_code_coverage.sh
fi

