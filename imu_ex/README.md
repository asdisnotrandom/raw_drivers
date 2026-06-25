MPU6050 için basit, filtresiz verileri USB-CDC protokolüyle herangi bir usb bağlantısı üzerinden terminal gibi bir ekrana taşıyabilecek, bu sayede debug veya fikir edinmeyi kolaylaştıracak kod. 
# Eklenecekler:
### BufferedUart ile anabilgisayara aktarım
### Filtreleme ile öncelikle bir referans noktası ve titreşimlere karşı dayanıklılık.
### CSV dosyalama ile optimizasyon
### (Belki)Health check (WHO_AM_I), zaman (timestamp), verinin ortalanması ve paketlenmesi
### beklenen çıktı 
### embassy-rp odaklı tasarımdansa, embedded-hal-async kütüphanesiyle evrensel yazım sağlanacak.
MPU6050 health_check (sadece en başta, ana bilgisayara hem sensörü hem sağlığını bildirecek)
zaman_damgasi,x,y,z,gx,gy,gz



