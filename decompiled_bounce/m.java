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
public final class m
extends k {
    byte a;
    private int c = 10;
    private int d;
    private int r;
    private int s;
    private int t;
    private int u;
    private byte e;
    private boolean b;
    private int v;
    private int w;
    int a;
    int b;
    private byte f;
    boolean a;
    byte d;
    private static int x = 2000;
    private static int[] a = new int[]{6000, 4000, 4000, 10000};
    private static int[] b = new int[]{100, 50, 80, 150};
    private static int[] c = new int[]{50, 100, 100, 150};

    public final int a(byte[] byArray, int n2) {
        n2 = super.a(byArray, n2);
        this.c = ((byArray[n2++] << 8 | byArray[n2++] & 0xFF) << 16) + this.c.c;
        this.d = ((byArray[n2++] << 8 | byArray[n2++] & 0xFF) << 16) + this.c.f;
        this.r = ((byArray[n2++] << 8 | byArray[n2++] & 0xFF) << 16) + this.c.c;
        this.s = ((byArray[n2++] << 8 | byArray[n2++] & 0xFF) << 16) + this.c.f;
        this.a = byArray[n2++];
        this.e = 0;
        this.v = 0;
        this.a = 0;
        this.b = false;
        if (this.c < this.r) {
            this.b = true;
        }
        this.d = 1;
        this.w = 2000;
        this.f = 1;
        this.b = 800;
        this.a = false;
        if (this.a == 3) {
            this.v = x;
            this.t = this.c.c;
            this.u = this.c.f;
            this.c = this.t;
            this.d = this.u;
            this.r = this.t;
            this.s = this.u;
        }
        return n2;
    }

    public final void a() {
        this.e = (byte)(-c[this.a] << 15);
        this.g = c[this.a] << 15;
        this.f = 0;
        this.h = b[this.a] << 16;
    }

    public final void b() {
        this.a(k.a);
        int n2 = k.a.c;
        int n3 = n.a.c.c;
        if (this.v <= 0 && this.d == 0 && this.f == 1) {
            this.v = 500;
            if (n2 < n3) {
                n.a.f += 200.0f;
                n.a.g += 400.0f;
            } else {
                n.a.f += -200.0f;
                n.a.g += 400.0f;
            }
            n.a.a = 0.0f;
            n.a.b = 0.0f;
            return;
        }
        if (this.d == 1) {
            n.a.f = n2 < n3 ? (n.a.f += 100.0f) : (n.a.f += -100.0f);
            n.a.a = 0.0f;
            n.a.b = 0.0f;
        }
    }

    public final void c() {
        if (n.f == 13 && g.a[8] == 5) {
            return;
        }
        switch (this.a) {
            case 0: {
                if (this.v > 0) break;
                this.a(k.a);
                int n2 = k.a.c;
                int n3 = n.a.c.c;
                n.a.f = n2 < n3 ? (n.a.f += 500.0f) : (n.a.f += -500.0f);
                n.a.a = 0.0f;
                n.a.b = 0.0f;
                this.v = 500;
                return;
            }
            case 1: {
                if (n.a.d == 1) {
                    this.f();
                    return;
                }
                g.a[0] = 1;
                return;
            }
            case 2: {
                if (this.v > 0 || this.d != 0 || this.f != 0) break;
                this.a = 0;
                this.f = 1;
                this.b = 200;
                return;
            }
            case 3: {
                g.a[0] = 1;
                this.c.c = this.t;
                this.c.f = this.u;
                this.c = this.t;
                this.d = this.u;
                this.r = this.t;
                this.s = this.u;
            }
        }
    }

    private void f() {
        this.a(k.a);
        int n2 = k.a.c;
        int n3 = k.a.f;
        n.g.a(10, n2, n3 + (b[this.a] << 15), 370, 0, 920, 230);
        this.j();
        n.a.c.c = n2;
        n.a.c.f = n3 + 0x1E0000;
        n.a.b.a((d)this.c);
        n.a.k();
        n.a.b.c(n.a.d);
    }

    public final void e() {
        switch (this.a) {
            case 0: {
                m.g();
                this.f();
                return;
            }
            case 1: 
            case 3: {
                return;
            }
            case 2: {
                m.g();
                this.f();
            }
        }
    }

    private static void g() {
        if (n.a.b < 0.0f) {
            n.a.b = -n.a.b * 0.5f;
            n.a.a *= 0.7f;
            return;
        }
        n.a.a = -n.a.a * 0.5f;
        n.a.b *= 0.7f;
    }

    public final void d() {
        double d2;
        int n2;
        int n3;
        int n4;
        int n5;
        this.h();
        super.d();
        if (n.f == 13 && g.a[8] == 5) {
            return;
        }
        int n6 = p.a;
        int n7 = this.c.c;
        int n8 = this.c.f;
        if (this.v > 0) {
            this.v -= n6;
            if (this.v <= 0) {
                this.v = 0;
                if (this.a == 3) {
                    this.v = x;
                    this.r = this.c;
                    this.s = this.d;
                    n5 = (x << 1) * a[this.a];
                    n4 = this.r - n7;
                    float f2 = n4 >> 16;
                    n3 = this.s - n8;
                    float f3 = n3 >> 16;
                    n2 = (int)(Math.sqrt(f2 * f2 + f3 * f3) * 65536.0);
                    if (n5 > n2) {
                        d2 = (double)n5 / (double)n2;
                        this.r += (int)((double)n4 * (d2 - 1.0));
                        this.s += (int)((double)n3 * (d2 - 1.0));
                    }
                    this.c = n.a.c.c;
                    this.d = n.a.c.f;
                }
            }
        }
        if (this.a == 3) {
            n4 = this.r;
            int n9 = n4 - n7;
            float f4 = n9 >> 16;
            n3 = this.s;
            int n10 = n3 - n8;
            float f5 = n10 >> 16;
            int n11 = (int)(Math.sqrt(f4 * f4 + f5 * f5) * 65536.0);
            if (n11 != 0) {
                int n12 = n6 * a[this.a];
                d2 = (double)n12 / (double)n11;
                n7 += (int)((double)n9 * d2);
                n8 += (int)((double)n10 * d2);
            }
        } else {
            if (this.a == 2) {
                if (this.w > 0) {
                    this.w -= n6;
                    if (this.w <= 0) {
                        if (this.d == 0) {
                            this.d = 1;
                            this.w = 2000;
                            this.f = 1;
                            this.b = 800;
                        } else {
                            this.d = 0;
                            this.w = 5000;
                            this.f = (byte)2;
                            this.b = 800;
                            this.a = true;
                        }
                    }
                }
                if (this.f == 1) {
                    this.a += n6;
                    if (this.a >= this.b) {
                        if (this.d == 0) {
                            this.f = (byte)2;
                            this.b = 200;
                        }
                        this.a = this.b;
                    }
                } else if (this.f == 2) {
                    this.a -= n6;
                    if (this.a <= 0) {
                        this.f = 0;
                        this.a = 0;
                        this.a = false;
                    }
                }
            }
            if (this.e == 0) {
                n4 = this.c;
                n3 = this.d;
                n2 = this.r;
            } else {
                n4 = this.r;
                n3 = this.s;
                n2 = this.c;
            }
            if (this.a == 0 && Math.abs(n7 - n4) > 0x460000 && Math.abs(n7 - n2) > 0x460000) {
                this.a(k.a);
                n5 = k.a.c;
                n2 = n.a.c.c;
                if (Math.abs(n2 - n5) < 0x780000 && (n5 < n2 && n4 < n7 || n5 > n2 && n4 > n7)) {
                    if (this.e == 0) {
                        n4 = this.r;
                        n3 = this.s;
                        this.e = 1;
                    } else {
                        n4 = this.c;
                        n3 = this.d;
                        this.e = 0;
                    }
                }
            }
            int n13 = n4 - n7;
            int n14 = n3 - n8;
            float f6 = n13 >> 16;
            float f7 = n14 >> 16;
            int n15 = (int)(Math.sqrt(f6 * f6 + f7 * f7) * 65536.0);
            boolean bl = false;
            boolean bl2 = false;
            if (n15 != 0) {
                double d3 = (double)(n6 *= a[this.a]) / (double)n15;
                n13 = (int)((double)n13 * d3);
                n14 = (int)((double)n14 * d3);
                n8 += n14;
                if (n13 >= 0 && (n7 += n13) >= n4 || n13 <= 0 && n7 <= n4) {
                    n7 = n4;
                    bl = true;
                }
                if (n14 >= 0 && n8 >= n3 || n14 <= 0 && n8 <= n3) {
                    n8 = n3;
                    bl2 = true;
                }
            }
            if (bl && bl2 || n15 == 0) {
                n7 = n4;
                n8 = n3;
                this.e = this.e == 0 ? (byte)1 : 0;
            }
        }
        if (this.a != 2 || this.a == 2 && this.d == 0 && this.f == 0) {
            this.b = true;
            if (this.c.c < n7) {
                this.b = false;
            }
            this.c.c = n7;
            this.c.f = n8;
            this.e = 1;
            this.f = 1;
        }
    }

    public final void a(Graphics graphics, DirectGraphics directGraphics, d d2) {
        this.a(k.a);
        d.a(d2, k.a, d.a);
        int n2 = d.a.c >> 16;
        int n3 = d.a.f >> 16;
        switch (this.a) {
            case 0: {
                int n4 = n.b / 50 % 6;
                p.b(n2, n3, 467, n4);
                return;
            }
            case 1: {
                if (this.b) {
                    p.a(n2, n3, -41);
                    return;
                }
                p.a(n2, n3, -42);
                return;
            }
            case 2: {
                if (this.f != 0) {
                    int n5 = graphics.getClipX();
                    int n6 = graphics.getClipY();
                    int n7 = graphics.getClipWidth();
                    int n8 = n3 - n6;
                    int n9 = graphics.getClipHeight();
                    if (n8 > n9) {
                        n8 = n9;
                    }
                    graphics.setClip(n5, n6, n7, n8);
                    n8 = this.a * 61 / this.b;
                    if (this.b) {
                        p.a(n2, n3 + 61 - n8, 189);
                    } else {
                        p.a(n2, n3 + 61 - n8, 194);
                    }
                    graphics.setClip(n5, n6, n7, n9);
                }
                int n10 = this.f == 0 ? n.a / 150 % 4 : 0;
                p.b(n2, n3, 504, n10);
                return;
            }
            case 3: {
                p.a(n2, n3, -28);
            }
        }
    }
}

