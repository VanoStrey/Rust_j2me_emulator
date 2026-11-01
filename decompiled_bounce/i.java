/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  com.nokia.mid.ui.DirectGraphics
 *  javax.microedition.lcdui.Graphics
 */
import com.nokia.mid.ui.DirectGraphics;
import javax.microedition.lcdui.Graphics;

/*
 * Duplicate member names - consider using --renamedupmembers true
 */
public final class i
extends k {
    private int c;
    int a;
    short a;
    private short d;
    short b;
    short c;
    private short e;
    private byte[] a;
    private int[] a;
    private int[] b;
    private byte[] b;
    private byte[] c;
    private int[] c = 6;
    private int[] d;
    private byte a = 0;
    private byte d;
    private byte e;
    private byte f;
    private int d;
    private int r;
    private int s = 0;
    private int t;
    private int u;
    int b;
    private byte g;

    public final int a(byte[] byArray, int n2) {
        n2 = super.a(byArray, n2);
        this.a = (short)(byArray[n2++] << 8 | byArray[n2++] & 0xFF);
        this.d = (short)(byArray[n2++] << 8 | byArray[n2++] & 0xFF);
        this.b = (short)(byArray[n2++] << 8 | byArray[n2++] & 0xFF);
        this.c = (short)(byArray[n2++] << 8 | byArray[n2++] & 0xFF);
        this.a = byArray[n2++];
        this.d = byArray[n2++];
        this.e = byArray[n2++];
        this.f = byArray[n2++];
        int n3 = ++n2;
        int n4 = ++n2;
        int n5 = ++n2;
        ++n2;
        int n6 = byArray[n5] & 0xFF;
        this.d = 0x44000000 | (byArray[n3] & 0xFF) << 16 | (byArray[n4] & 0xFF) << 8 | n6;
        this.g = n6 == 16 ? (byte)1 : 0;
        if (this.d != 0x44000000) {
            this.d = 0x441111EE;
        }
        this.e = (short)(this.b - this.a);
        if (this.d != 0x44000000 && this.g == 0) {
            int n7;
            n6 = this.e * 50 / 100;
            this.a = new byte[n6];
            this.r = n6 << 12;
            this.u = 0;
            this.a = n6 + 2;
            for (n7 = 0; n7 < n6; ++n7) {
                this.a[n7] = 0;
            }
            for (n7 = 0; n7 < this.c; ++n7) {
                this.a[n7] = 0;
            }
            this.c = ((n6 /= 20) << 1) + 2;
            this.a = new int[this.c];
            this.b = new int[this.c];
            this.b = new byte[this.c];
            this.c = new byte[this.c];
            this.c = new int[this.c];
            this.d = new int[this.c];
            for (n7 = 2; n7 <= n6 - 2; ++n7) {
                int n8 = (this.a.length - 1 << 12) * n7 / n6;
                int n9 = (Math.abs(n.a.nextInt() % 2) + 24 << 12 << 1) / 9;
                int n10 = (Math.abs(n.a.nextInt() % 2) + 10 << 12) / 3;
                int n11 = n9 * 3;
                this.a(n9, n11, -1, n10, n8, 1);
                n9 = (Math.abs(n.a.nextInt() % 2) + 24 << 12 << 1) / 9;
                n10 = (Math.abs(n.a.nextInt() % 2) + 10 << 12) / 3;
                n11 = n9 * 3;
                this.a(n9, n11, 1, n10, n8, 1);
            }
        }
        return n2;
    }

    public final void a() {
        this.e = (short)(this.a << 16);
        this.g = (byte)(this.b << 16);
        this.f = (byte)((this.d << 16) - 0x1E0000);
        this.h = (this.c << 16) + 0x1E0000;
        this.b = this.c << 16;
    }

    public final void a(int n2, float f2, int n3, c c2) {
        if (this.d != 0x44000000) {
            int n4;
            int n5;
            this.t = 0;
            float f3 = 230.0f;
            if (f2 > f3) {
                f2 = f3;
            } else if (f2 < -f3) {
                f2 = -f3;
            }
            int n6 = (int)Math.abs(f2);
            if (this.g == 0) {
                int n7 = this.a.length - 1 << 12;
                n2 = (int)((long)n7 * (long)n2 / 65536L);
                n5 = (n6 * 100 << 12) / 1500;
                n4 = n5 << 1;
                this.a(n5, n4, -1, 20480, n2 - (n3 << 2), 0);
                this.a(n5, n4, 1, 20480, n2 + (n3 << 2), 0);
            }
            c2.j /= 3.0f;
            c2.k /= 3.0f;
            if (this.g == 0) {
                this.a(k.a);
                int n8 = k.a.f + (this.c << 16);
                n2 = n6 * 500 / 230;
                n5 = n6 * 6 / 230;
                n4 = n6 * 800 / 230;
                n.b.a(n5, n.a.c.c, n8, n2, n2 >> 2, 325, 30, n4, n4 / 6);
                n.b.a(n5, n.a.c.c, n8, n2, n2 >> 2, 35, 30, n4, n4 / 6);
            }
        }
    }

    public final void a(int n2, int n3, c c2) {
        int n4;
        int n5;
        int n6;
        this.t += p.a;
        if (n.a) {
            return;
        }
        float f2 = (float)(n2 - this.e) / (float)(this.g - this.e);
        float f3 = (float)(n3 - this.f) / (float)(this.h - this.f);
        if (this.f < this.d) {
            n6 = this.d - this.f;
            n5 = this.f + (int)(f2 * (float)n6);
        } else {
            n6 = this.f - this.d;
            n5 = this.d + (int)((1.0f - f2) * (float)n6);
        }
        if (this.e < this.a) {
            n6 = this.a - this.e;
            n4 = this.e + (int)(f3 * (float)n6);
        } else {
            n6 = this.e - this.a;
            n4 = this.a + (int)((1.0f - f3) * (float)n6);
        }
        c2.d += (float)(n5 << 5);
        c2.e += (float)(n4 << 5);
        if (c2.j > 0.0f) {
            c2.j -= (float)p.a * c2.j / (float)400;
            if (c2.j < 0.0f) {
                c2.j = 0.0f;
            }
        } else if (c2.j < 0.0f) {
            c2.j -= (float)p.a * c2.j / (float)400;
            if (c2.j > 0.0f) {
                c2.j = 0.0f;
            }
        }
        if (c2.k > 0.0f) {
            c2.k -= (float)p.a * c2.k / (float)400;
            if (c2.k < 0.0f) {
                c2.k = 0.0f;
            }
        } else if (c2.k < 0.0f) {
            c2.k -= (float)p.a * c2.k / (float)400;
            if (c2.k > 0.0f) {
                c2.k = 0.0f;
            }
        }
        if (this.d != 0x44000000) {
            float f4 = 1.0f / c.a[n.a.d];
            float f5 = (float)p.a * 0.0014f;
            if (n.a.a > 0.0f) {
                n.a.a -= n.a.a * f4 * f5;
                if (n.a.a < 0.0f) {
                    n.a.a = 0.0f;
                }
            } else {
                n.a.a -= n.a.a * f4 * f5;
                if (n.a.a > 0.0f) {
                    n.a.a = 0.0f;
                }
            }
            if (n.a.b > 0.0f) {
                n.a.b -= n.a.b * f4 * f5;
                if (n.a.b < 0.0f) {
                    n.a.b = 0.0f;
                }
            } else {
                n.a.b -= n.a.b * f4 * f5;
                if (n.a.b > 0.0f) {
                    n.a.b = 0.0f;
                }
            }
        }
        if (this.t > 150 && this.d != 0x44000000) {
            int n7;
            this.a(k.a);
            n.c.b = n7 = k.a.f + (this.c << 16);
            n.c.c = 60000;
            int n8 = g.a[4] / 60;
            n.c.b(n8, n.a.c.c, n.a.c.f, c.a[0] << 15, 0, 0, 0, 0, 4000, 666);
            this.t = 0;
        }
        if (this.d != 0x44000000) {
            n.a.b = (float[])8;
        }
        n.a = true;
    }

    private void a(int n2, int n3, int n4, int n5, int n6, int n7) {
        for (int i2 = 0; i2 < this.c; ++i2) {
            if (this.a[i2] != 0) continue;
            this.a[i2] = n2 * 50 / 100;
            this.b[i2] = n3 * 50 / 100;
            this.b[i2] = (byte)n4;
            this.c[i2] = (byte)n7;
            this.c[i2] = n5 * 50 / 100;
            this.d[i2] = n6;
            return;
        }
    }

    /*
     * Enabled aggressive block sorting
     */
    public final void a(Graphics graphics, DirectGraphics directGraphics, d d2) {
        int n2;
        int n3;
        int n4;
        int n5;
        int n6;
        int n7;
        block33: {
            int n8;
            block34: {
                int n9;
                block32: {
                    n8 = p.a * p.a();
                    if (this.d == 0x44000000) break block32;
                    if (!n.b) {
                        this.u += n8;
                        if (this.u > 150) {
                            this.a(k.a);
                            n7 = k.a.c + (this.a << 16);
                            n6 = k.a.c + (this.b << 16);
                            n5 = k.a.f + (this.d << 16);
                            n.c.b = n4 = k.a.f + (this.c << 16);
                            n.c.c = 60000;
                            n.c.a(1, n7, n5, n6, n5, 0, 0, 0, 0, 4000, 666);
                            this.u = 0;
                        }
                    }
                    if (n.b || this.g != 0) break block33;
                    this.s += n8;
                    for (n7 = 0; n7 < this.a.length; ++n7) {
                        this.a[n7] = 0;
                    }
                    break block34;
                }
                if (n.b) return;
                this.u += n8;
                if (this.u <= 150) return;
                this.a(k.a);
                int n10 = k.a.c + (this.a << 16);
                int n11 = k.a.c + (this.b << 16);
                int n12 = k.a.f + (this.d << 16);
                int n13 = k.a.f + (this.c << 16);
                int n14 = n10;
                n8 = n11;
                int n15 = n12;
                int n16 = n13;
                int n17 = Math.abs(this.f);
                if (Math.abs(this.d) > n17) {
                    n17 = Math.abs(this.d);
                }
                int n18 = Math.abs(this.a);
                if (Math.abs(this.e) > n18) {
                    n18 = Math.abs(this.e);
                }
                if (this.f + this.d > 0) {
                    n17 <<= 7;
                    n9 = 90;
                    n8 = n10;
                } else if (this.f + this.d < 0) {
                    n17 <<= 7;
                    n9 = 270;
                    n14 = n11;
                } else if (this.a + this.e > 0) {
                    n17 = n18 << 7;
                    n9 = 0;
                    n16 = n12;
                } else {
                    n17 = n18 << 7;
                    n9 = 180;
                    n15 = n13;
                }
                n.i.a(1, n14, n15, n8, n16, n17, n17 / 8, n9, 0, 2000, 333);
                this.u = 0;
                return;
            }
            for (n7 = 0; n7 < this.c; ++n7) {
                if (this.a[n7] == 0) continue;
                if (this.c[n7] != 1) {
                    int n19 = n7;
                    this.a[n19] = this.a[n19] - (n8 * 28 >> 3);
                    int n20 = n7;
                    this.b[n20] = this.b[n20] - (n8 * 12 >> 3);
                    int n21 = n7;
                    this.c[n21] = this.c[n21] - (n8 * 4 >> 3);
                }
                int n22 = n7;
                this.d[n22] = this.d[n22] + (this.b[n7] * this.c[n7] * n8 >> 6);
                if (this.b[n7] <= 0 || this.c[n7] <= 0) {
                    this.a[n7] = 0;
                }
                if (this.a[n7] <= 0) {
                    this.a[n7] = 0;
                } else if (this.d[n7] < 0 || this.d[n7] >> 12 >= this.r >> 12) {
                    int n23 = n7;
                    this.b[n23] = -this.b[n23];
                    if (this.d[n7] < 0) {
                        this.d[n7] = 0;
                    } else if (this.d[n7] >> 12 >= this.r >> 12) {
                        this.d[n7] = (this.r >> 12) - 1 << 12;
                    }
                    if (this.c[n7] != 1) {
                        int n24 = n7;
                        this.a[n24] = this.a[n24] - (28 * (n8 << 1) >> 3);
                        int n25 = n7;
                        this.b[n25] = this.b[n25] - (12 * (n8 << 1) >> 3);
                        int n26 = n7;
                        this.c[n26] = this.c[n26] - (4 * (n8 << 1) >> 3);
                    }
                }
                if (this.a[n7] <= 0 || (n6 = this.d[n7] >> 12) < 0 || n6 >= this.a.length) continue;
                int n27 = n6;
                this.a[n27] = (byte)(this.a[n27] + (this.a[n7] >> 12));
                n4 = this.b[n7] >> 12;
                if (n4 == 0) {
                    n4 = 1;
                }
                n3 = 180 / n4;
                for (n2 = 1; n2 < n4; ++n2) {
                    int n28 = this.a[n7] * (n.b[90 + n3 * n2] + 360 >> 1) / 360 >> 12;
                    if (n6 - n2 >= 0) {
                        int n29 = n6 - n2;
                        this.a[n29] = (byte)(this.a[n29] + n28);
                    }
                    if (n6 + n2 >= this.a.length) continue;
                    int n30 = n6 + n2;
                    this.a[n30] = (byte)(this.a[n30] + n28);
                }
            }
            for (n7 = 0; n7 < this.a.length; ++n7) {
                if (n7 <= 0 || this.a[n7 - 1] >= this.a[n7] || n7 >= this.a.length - 1 || this.a[n7 + 1] >= this.a[n7]) continue;
                int n31 = n7;
                this.a[n31] = (byte)(this.a[n31] - 1);
            }
        }
        this.a(k.a);
        d.a(d2, k.a, d.a);
        d.a.a(this.a << 16, this.c << 16);
        n7 = d.g >> 16;
        n6 = d.h >> 16;
        d.a.a(this.b << 16, this.d << 16);
        n5 = d.g >> 16;
        n4 = d.h >> 16;
        n3 = n5 - n7;
        if (this.g != 0) {
            n2 = n.c(this.d);
            int[] nArray = q.a;
            int[] nArray2 = q.b;
            nArray[0] = n7;
            nArray2[0] = n6;
            nArray[1] = n5;
            nArray2[1] = n6;
            nArray[2] = n5;
            nArray2[2] = n4;
            nArray[3] = n7;
            nArray2[3] = n4;
            directGraphics.fillPolygon(nArray, 0, nArray2, 0, 4, n2);
            return;
        }
        n2 = (n3 << 8 << 10) / this.a.length >> 8;
        int[] nArray = q.a;
        int[] nArray3 = q.b;
        nArray[0] = n5;
        nArray3[0] = n4;
        nArray[1] = n7;
        nArray3[1] = n4;
        int n32 = 2;
        int n33 = 2;
        while (true) {
            if (n33 >= this.a.length + 2) {
                n33 = n.c(this.d);
                directGraphics.fillPolygon(nArray, 0, nArray3, 0, n32, n33);
                return;
            }
            if (n33 > 2) {
                n3 = this.a[n33 - 2];
                int n34 = 0;
                for (n4 = n33 + 1; n4 < this.a.length + 2 && this.a[n4 - 2] == n3; ++n34, ++n4) {
                }
                n33 += n34;
            }
            if (n33 == this.a.length + 1) {
                nArray[n32] = n5;
                nArray3[n32] = n6;
            } else {
                nArray[n32] = ((n33 - 2) * n2 >> 10) + n7;
                nArray3[n32] = (-this.a[n33 - 2] * n2 >> 10) + n6;
            }
            ++n32;
            ++n33;
        }
    }
}

