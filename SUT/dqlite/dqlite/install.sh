autoreconf -i && \ 
./configure --enable-debug --enable-sanitize && \ 
make -j$(getconf _NPROCESSORS_ONLN) && \
make install