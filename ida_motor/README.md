# PIN LAYOUT
pin0, pin2, pin4, pin6 ya baglanmis 4 ayri dc motoru; pico 2 ile yuzdelik olarak gonderilen sayiyi parse ederek pwm sinyallerine iletme.
GOnderilmesi gereken format => [i32,i32,i32,i32]
sirayla pin0, pin2, pin4, pin6 ya bagli motorlara pwm sinyalini gonderir.

# Eklenmesi gerekenler
defmt debug ortamina gecis
PID ile daha stabil veri isleme (PI4 ile pid yaparak da sadece yuzdelik veriler gonderilebilir ki daha mantikli bir tercih olabildigi gibi, piconun gucu PID hesaplamalarina yetecek kadar guclu ve ustundeki islem yuku su an fazlasiyla az)

# Ozellikler
acil durumda motor pwmleri sifirlanir yani arac durur.
hata durumunda debug edilebilir bir yapi, terminalden okuma yapilmali
motorlarin gucu picodan saglanmamali, kaldiramaz
micro usb - usb ile baglanmali, usb serial cdcacm class iletisim protokolu.