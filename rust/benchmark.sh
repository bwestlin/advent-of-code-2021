
CPU_MODEL=`cat /proc/cpuinfo | grep 'model name' | head -n 1 | cut -d ':' -f2 | sed -e 's/^[[:space:]]*//'`

echo "**CPU:** $CPU_MODEL"
echo

echo "| Day | Least runtime both parts |"
echo "| --- | ------------------------:|"

 for DAY in {01..25} ; do

    RESULT=`./run.sh $DAY timeit 2>&1 | grep 'It took' | cut -d ' ' -f3 | sed -e 's/^[[:space:]]*//'`
    echo "| $DAY | $RESULT |";

done