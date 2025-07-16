autoreconf -i  && \
./configure --enable-debug --enable-sanitize &&
make -j$(getconf _NPROCESSORS_ONLN) && \
make install
cd ../dqlite
autoreconf -i && \ 
./configure --enable-debug --enable-sanitize && \ 
make -j$(getconf _NPROCESSORS_ONLN) && \
make install