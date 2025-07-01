autoreconf -i  && \
sudo ./configure --enable-debug --enable-sanitize 
sudo make -j$(getconf _NPROCESSORS_ONLN) && \
sudo make install
cd ../dqlite
autoreconf -i && \ 
sudo ./configure --enable-debug --enable-sanitize && \ 
sudo make -j$(getconf _NPROCESSORS_ONLN) && \
sudo make install