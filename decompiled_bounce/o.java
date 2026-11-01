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
public final class o
extends k {
    private static boolean a = false;
    private int[] a;
    private int[] b;
    private int[] c;
    private int[] d;
    private int[] e;
    private short[] a;
    private byte[] a;
    private byte[] b;
    private int d;
    private int r;
    private int s;
    private int t;
    private int u;
    public int a;
    private int v;
    private int w;
    public int b;
    public int c;
    private int[] f;
    private int x;
    private static int[] g = new int[2];

    public o(int n2, int n3, int n4, int n5, int n6, int n7, int n8, int[] nArray, int n9, int n10) {
        if (!a) {
            a = true;
        }
        this.a = new int[n2];
        this.b = new int[n2];
        this.c = new int[n2];
        this.d = new int[n2];
        this.e = new int[n2];
        this.b = new byte[n2];
        if (n8 == 4) {
            this.a = new short[n2];
            this.a = new byte[n2];
            for (n3 = 0; n3 < n2; ++n3) {
                this.a[n3] = 0;
            }
        }
        this.v = n2;
        this.d = 0;
        this.r = n4;
        this.s = 0;
        this.t = 0;
        this.u = n7;
        this.a = -1;
        this.w = n8;
        this.c = (int[])11;
        this.f = nArray;
        this.x = n9;
        this.b = (int[])((byte)n10);
    }

    public final void c(k k2) {
        this.b(k2);
        this.c.a = 65536;
        this.c.b = 0;
        this.c.c = 0;
        this.c.d = 0;
        this.c.e = 65536;
        this.c.f = 0;
        this.b.a((d)this.c);
        this.a();
    }

    public final void a(int n2, int n3, int n4, int n5, int n6, int n7, int n8) {
        if (n2 + 1 + this.a > this.v) {
            n2 = this.v - this.a - 1;
        }
        for (int i2 = 0; i2 < n2; ++i2) {
            n6 = i2 * 360 / n2;
            int n9 = n6 + 90 >= 360 ? n6 + 90 - 360 : n6 + 90;
            ++this.a;
            this.a[this.a] = n8 != 0 ? n7 + n.a.nextInt() % n8 : n7;
            this.b[this.a] = n3;
            this.c[this.a] = n4;
            this.d[this.a] = n5 * n.b[n6];
            this.e[this.a] = n5 * n.b[n9];
            if (this.w == 3) {
                this.b[this.a] = (byte)(i2 % this.f.length);
                continue;
            }
            if (this.w == 4 && this.equals(n.a)) {
                byte by = (byte)Math.abs(n.a.nextInt() % this.f.length);
                n6 = by;
                if (by > 1) {
                    n6 = (byte)Math.abs(n.a.nextInt() % this.f.length);
                }
                this.b[this.a] = n6;
                continue;
            }
            this.b[this.a] = (byte)Math.abs(n.a.nextInt() % this.f.length);
        }
    }

    public final void a(int n2, int n3, int n4, int n5, int n6, int n7, int n8, int n9, int n10) {
        if (n2 + 1 + this.a > this.v) {
            n2 = this.v - this.a - 1;
        }
        for (int i2 = 0; i2 < n2; ++i2) {
            int n11;
            int n12 = (n11 += (n11 -= (n11 = n7 + n.a.nextInt() % (n8 / 2 + 1)) >= 360 ? 360 : 0) < 0 ? 360 : 0) + 90 >= 360 ? n11 - 270 : n11 + 90;
            int n13 = n6 != 0 ? n5 + n.a.nextInt() % n6 : n5;
            ++this.a;
            this.a[this.a] = n10 != 0 ? n9 + n.a.nextInt() % n10 : n9;
            this.b[this.a] = n3;
            this.c[this.a] = n4;
            this.d[this.a] = n13 * n.b[n11];
            this.e[this.a] = n13 * n.b[n12];
            if (this.w == 0) {
                if (n11 >= 35 && n11 <= 90) {
                    this.b[this.a] = 2;
                    continue;
                }
                if (n11 < 35) {
                    this.b[this.a] = 3;
                    continue;
                }
                if (n11 > 325) {
                    this.b[this.a] = 1;
                    continue;
                }
                this.b[this.a] = 0;
                continue;
            }
            if (this.w == 7) {
                if (n7 == 0 || n7 == 180) {
                    this.b[this.a] = 0;
                    continue;
                }
                this.b[this.a] = 1;
                continue;
            }
            this.b[this.a] = (byte)Math.abs(n.a.nextInt() % this.f.length);
        }
    }

    public final void a(int n2, int n3, int n4, int n5, int n6, int n7, int n8, int n9, int n10, int n11) {
        if (11 + this.a > this.v) {
            n2 = this.v - this.a - 1;
        }
        for (n10 = 0; n10 < n2; ++n10) {
            n5 = 800 + n.a.nextInt() % 200;
            ++this.a;
            this.a[this.a] = 800 + n.a.nextInt() % 200;
            this.b[this.a] = n3;
            this.c[this.a] = n4;
            n6 = n7 + (n.a.nextInt() % 30 << 10);
            n9 = n8 + (n.a.nextInt() % 30 << 10);
            this.d[this.a] = n5 * n6 >> 8;
            this.e[this.a] = n5 * n9 >> 8;
            this.b[this.a] = (byte)Math.abs(n.a.nextInt() % this.f.length);
        }
    }

    public final void b(int n2, int n3, int n4, int n5, int n6, int n7, int n8, int n9, int n10, int n11) {
        for (n8 = 0; n8 < n2; ++n8) {
            while ((n6 = n.a.nextInt() % (n5 * 2 + 1)) * n6 + (n7 = n.a.nextInt() % (n5 * 2 + 1)) * n7 > n5 * n5) {
            }
            n6 = n3 + n6;
            n7 = n4 + n7;
            this.a(1, n6, n7, 0, 0, 0, 0, n10, n11);
        }
    }

    public final void a(int n2, int n3, int n4, int n5, int n6, int n7, int n8, int n9, int n10, int n11, int n12) {
        for (int i2 = 0; i2 < n2; ++i2) {
            int n13 = n3 + Math.abs(n.a.nextInt() % (n5 - n3 + 1));
            int n14 = n4 + Math.abs(n.a.nextInt() % (n6 - n4 + 1));
            this.a(1, n13, n14, n7, n8, n9, n10, n11, n12);
        }
    }

    public final void a(Graphics graphics, DirectGraphics directGraphics, d d2) {
        int n2 = p.a * p.a();
        for (int i2 = 0; i2 < this.a + 1; ++i2) {
            int n3;
            int n4 = 0;
            int n5 = i2;
            this.a[n5] = this.a[n5] - n2;
            if (this.a[i2] > 0) {
                if (this.u != 0) {
                    int n6 = i2;
                    this.d[n6] = this.d[n6] - (this.d[i2] * this.u * n2 >> 14);
                    int n7 = i2;
                    this.e[n7] = this.e[n7] - (this.e[i2] * this.u * n2 >> 14);
                }
                int n8 = i2;
                this.d[n8] = this.d[n8] + this.d * n2;
                int n9 = i2;
                this.e[n9] = this.e[n9] + this.r * n2;
                if (this.w == 1 && this.e[i2] > this.c) {
                    this.e[i2] = this.c;
                } else if (this.w == 4) {
                    int n10 = i2;
                    this.a[n10] = (short)(this.a[n10] - n2);
                    if (this.a[i2] <= 0) {
                        this.a[i2] = (short)(Math.abs(n.a.nextInt() % 200) + 300);
                        this.a[i2] = (byte)(Math.abs(n.a.nextInt()) & 3);
                    }
                    n3 = n2 / 2;
                    if (this.a[i2] == 1) {
                        o.a(n3, this.d[i2], this.e[i2]);
                        this.d[i2] = g[0];
                        this.e[i2] = g[1];
                    } else if (this.a[i2] == 2) {
                        o.a(359 - n3, this.d[i2], this.e[i2]);
                        this.d[i2] = g[0];
                        this.e[i2] = g[1];
                    }
                } else if (this.w == 5) {
                    n3 = n2 / 5;
                    o.a(n3, this.d[i2], this.e[i2]);
                    this.d[i2] = g[0];
                    this.e[i2] = g[1];
                }
                int n11 = i2;
                this.b[n11] = this.b[n11] + ((this.d[i2] + this.s) * n2 >> 4);
                int n12 = i2;
                this.c[n12] = this.c[n12] + ((this.e[i2] + this.t) * n2 >> 4);
            } else {
                n4 = 1;
            }
            if (this.w == 1 && this.c[i2] >= this.b) {
                n4 = 1;
            }
            if (n4 != 0) {
                if (i2 == this.a) {
                    --this.a;
                    continue;
                }
                this.a[i2] = this.a[this.a];
                this.b[i2] = this.b[this.a];
                this.c[i2] = this.c[this.a];
                this.d[i2] = this.d[this.a];
                this.e[i2] = this.e[this.a];
                --this.a;
                --i2;
                continue;
            }
            n3 = p.b(this.f[this.b[i2]]);
            n4 = this.a[i2] * n3 / this.x;
            if (n4 > n3 - 1) {
                n4 = n3 - 1;
            }
            n4 = n3 - 1 - n4;
            d2.a(this.b[i2], this.c[i2]);
            n3 = d.g >> 16;
            int n13 = d.h >> 16;
            p.b(n3, n13, this.f[this.b[i2]], n4);
        }
    }

    private static void a(int n2, int n3, int n4) {
        short s2 = n.b[n2 % 360];
        n2 = n.b[(n2 + 90) % 360];
        o.g[0] = (n3 * n2 - n4 * s2) / 360;
        o.g[1] = (n3 * s2 + n4 * n2) / 360;
    }
}

