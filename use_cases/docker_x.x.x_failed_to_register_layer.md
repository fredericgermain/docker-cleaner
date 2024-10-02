# Use Case 1/10/2024

Docker version 20.10.2, build 20.10.2-0ubuntu1~18.04.2
Docker Compose version v2.27.0

docker-compose -f docker-compose.yml up
[+] Running 30/1
 ⠇ frigate [⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿] 830.2MB / 836MB   Pulling                                                                                                                                47.8s 
failed to register layer: open /var/lib/docker/overlay2/cdb5b3e04b0ead64b09164330949eb373af03e322642cf1a0cc2fe39c61f3f4c/committed: no such file or directory

MissingNode:Overlay2:/cdb5b3e04b0ead64b09164330949eb373af03e322642cf1a0cc2fe39c61f3f4c => rdep:ImageLayer:9beca... ... > rdep Container:7f2c662f5
DiffId > ImageLayer > Missing overlay2
