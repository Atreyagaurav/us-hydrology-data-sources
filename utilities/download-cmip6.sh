#!/usr/bin/env bash
# the parameter to the script is the link for json result on https://aims2.llnl.gov/metagrid/search/?project=CMIP6
search_link="$1"
search_results=`curl -s $search_link`
dataset_ids=`echo $search_results | jq -r '.response.docs[].id'`

rm ./filesizes.txt
for data_id in $dataset_ids;
do
    files_query_res=`curl "https://aims2.llnl.gov/metagrid-backend/proxy/search?dataset_id=${data_id}&format=application%2Fsolr%2Bjson&limit=1000&offset=0&type=File&" -H 'User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:104.0) Gecko/20100101 Firefox/104.0' -H 'Accept: application/json, text/plain, */*' -H 'Accept-Language: en-US,en;q=0.5' -H 'Accept-Encoding: gzip, deflate, br' -H 'X-Requested-With: XMLHttpRequest' -H 'Connection: keep-alive' -H 'Referer: https://aims2.llnl.gov/search' -H 'Sec-Fetch-Dest: empty' -H 'Sec-Fetch-Mode: cors' -H 'Sec-Fetch-Site: same-origin' -H 'TE: trailers'`

    #   comment if you don't want file sizes.
    file_sizes=`echo $files_query_res | jq -r '.response.docs[].size'`
    for fs in $file_sizes;
    do
	echo $fs >> ./filesizes.txt;
    done;
    
    file_urls=`echo $files_query_res | jq -r '.response.docs[].url[0]' | awk -F\| '{ print $1 }'`
    for file_url in $file_urls;
    do
	echo $file_url;
    done;
done;
