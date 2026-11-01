/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  com.nokia.mid.ui.DirectGraphics
 *  javax.microedition.lcdui.Graphics
 */
import com.nokia.mid.ui.DirectGraphics;
import javax.microedition.lcdui.Graphics;

public final class s
extends k {
    private short[] c = (short[])5;
    private short[] d;
    short[] a;
    short[] b;

    public final int a(byte[] byArray, int n2) {
        n2 = super.a(byArray, n2);
        int n3 = byArray[n2++] & 0xFF;
        this.c = new short[n3];
        this.d = new short[n3];
        this.a = new short[n3];
        this.b = new short[n3];
        if (n3 > 0) {
            int n4 = byArray[n2++] & 0xFF;
            short s2 = (short)(byArray[n2++] << 8 | byArray[n2++] & 0xFF);
            short s3 = (short)(byArray[n2++] << 8 | byArray[n2++] & 0xFF);
            if (n4 > 0) {
                n2 = s.a(this.c, n3, s2, 1, byArray, n2, n4);
                n2 = s.a(this.d, n3, s3, 1, byArray, n2, n4);
            } else {
                for (n4 = 0; n4 < n3; ++n4) {
                    this.c[n4] = s2;
                    this.d[n4] = s3;
                }
            }
            n2 = s.a(this.a, n3, 0, 1, byArray, n2, 16);
            for (n4 = 0; n4 < n3; ++n4) {
                this.b[n4] = -1;
            }
        }
        return n2;
    }

    private static int a(int n2) {
        switch (n2) {
            case 485: {
                return 150;
            }
            case 474: {
                return 150;
            }
            case 480: {
                return 150;
            }
        }
        return 0;
    }

    public final void a() {
        super.a();
        for (int i2 = 0; i2 < this.a.length; ++i2) {
            int n2 = (p.a((int)this.a[i2], 0) << 16) / k.g.a;
            int n3 = (p.a((int)this.a[i2], 1) << 16) / k.g.a;
            int n4 = (p.a((int)this.a[i2], 2) << 16) / k.g.a;
            int n5 = (p.a((int)this.a[i2], 3) << 16) / k.g.a;
            n4 = this.c[i2] - n4;
            n2 = n4 + n2;
            n5 = this.d[i2] - (n3 - n5);
            n3 = n5 + n3;
            if (n4 < this.e) {
                this.e = n4;
            }
            if (n2 > this.g) {
                this.g = n2;
            }
            if (n5 < this.f) {
                this.f = n5;
            }
            if (n3 <= this.h) continue;
            this.h = n3;
        }
        this.e <<= 16;
        this.g <<= 16;
        this.f <<= 16;
        this.h <<= 16;
    }

    public final void d() {
        super.d();
        for (int i2 = 0; i2 < this.a.length; ++i2) {
            if (this.b[i2] <= -1) continue;
            int n2 = p.a;
            if (this.a[i2] != 9999) {
                int n3 = i2;
                this.a[n3] = (short)(this.a[n3] - n2);
            }
            if (this.a[i2] > 0) continue;
            if (this.b[i2] == 474) {
                if (this.a[i2] >= -1500) continue;
                this.j();
                return;
            }
            this.j();
            return;
        }
    }

    public final void a(Graphics graphics, DirectGraphics directGraphics, d d2) {
        this.a(k.a);
        d.a(d2, k.a, d.a);
        for (int i2 = 0; i2 < this.a.length; ++i2) {
            d.a.a(this.c[i2] << 16, this.d[i2] << 16);
            int n2 = d.g >> 16;
            int n3 = d.h >> 16;
            if (this.b[i2] > -1) {
                int n4 = 0;
                int n5 = this.a[i2] != 9999 ? this.a[i2] : n.a;
                if (this.a[i2] <= 0 && this.b[i2] == 474) {
                    n5 = 1;
                    n4 = (Math.abs(this.a[i2]) * 255 / 1500 & 0xFF) << 24;
                }
                int n6 = d.g >> 16;
                int n7 = d.h >> 16;
                Graphics graphics2 = p.a();
                if (n4 != 0) {
                    n.b.setColor(255);
                    n.b.fillRect(0, 0, n.b.getWidth(), n.b.getHeight());
                    n2 = n.b.getWidth() >> 1;
                    n3 = n.b.getHeight();
                    graphics = n.b;
                    p.a(graphics);
                }
                int n8 = p.b(this.b[i2]);
                if (this.a[i2] != 9999) {
                    int n9;
                    short s2 = this.b[i2];
                    switch (s2) {
                        case 480: 
                        case 485: {
                            n9 = 9999;
                            break;
                        }
                        case 474: {
                            n9 = 750;
                            break;
                        }
                        default: {
                            n9 = 0;
                        }
                    }
                    n5 = (n9 - n5) / s.a(this.b[i2]) % n8;
                } else {
                    n5 = n5 / s.a(this.b[i2]) % n8;
                }
                if (n5 > n8 - 1) {
                    n5 = n8 - 1;
                }
                p.b(n2, n3, this.b[i2], n5);
                if (n4 == 0) continue;
                p.a(graphics2);
                n.b.getRGB(n.b, 0, n.b.getWidth(), 0, 0, n.b.getWidth(), n.b.getHeight());
                n8 = 0;
                for (n2 = 0; n2 < n.b.getHeight(); ++n2) {
                    for (n3 = 0; n3 < n.b.getWidth(); ++n3) {
                        if (n.b[n8] == -16776961) {
                            n.b[n8] = 0;
                        } else {
                            int n10 = n8;
                            n.b[n10] = n.b[n10] - n4;
                        }
                        ++n8;
                    }
                }
                n2 = n6 - (n.b.getWidth() >> 1);
                n3 = n7 - n.b.getHeight();
                p.a().drawRGB(n.b, 0, n.b.getWidth(), n2, n3, n.b.getWidth(), n.b.getHeight(), true);
                continue;
            }
            p.a(n2, n3, (int)this.a[i2]);
        }
    }
}

