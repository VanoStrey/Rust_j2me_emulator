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
public final class l
extends k {
    private short a;
    private byte a;
    private int r;
    private boolean a;
    private int s;
    int a;
    int b;
    int c = 7;
    int d;
    private static int[] a = new int[]{500, 100, 300};
    private static int t = a[0] + a[1] + a[2];
    private static int u = a[2];

    public final int a(byte[] byArray, int n2) {
        n2 = super.a(byArray, n2);
        this.a = (short)(byArray[n2++] << 8 | byArray[n2++] & 0xFF);
        this.a = byArray[n2++];
        this.r = 0;
        this.a = false;
        this.s = 0;
        if (this.c.a >= 0) {
            this.a = true;
        }
        return n2;
    }

    public final void a() {
        this.e = -7864320;
        this.g = 0x780000;
        this.f = -7864320;
        this.h = 0x780000;
        this.a = -2621440;
        this.b = 0x280000;
        this.c = -2621440;
        this.d = 0x280000;
    }

    public final void b() {
        if (this.r == 0 && g.a[1] == 0) {
            g.a[1] = 1;
            n.a = this;
            c c2 = (c)this.a().a(this.a);
            this.a(k.a);
            c2.a(k.a.c, k.a.f + 0x230000);
            c2.a = false;
            c2.b = false;
        }
    }

    public final void c() {
        if (this.s == 0) {
            float f2 = (float)p.a * 0.001f;
            f2 = 3.0f * f2;
            d.a.a(f2);
            d.a.c = 0;
            d.a.f = 0;
            this.c.b(d.a);
            if (this.a && this.c.a < 0) {
                this.c.a = 0;
                this.c.d = 65536;
                this.c.b = -65536;
                this.c.e = 0;
            }
            if (!this.a && this.c.a > 0) {
                this.c.a = 0;
                this.c.d = 65536;
                this.c.b = 65536;
                this.c.e = 0;
            }
            this.h();
        }
    }

    public final void e() {
        if (this.s == 0) {
            float f2 = (float)p.a * 0.001f;
            f2 = -3.0f * f2;
            d.a.a(f2);
            d.a.c = 0;
            d.a.f = 0;
            this.c.b(d.a);
            if (this.a && this.c.b > 0) {
                this.c.a = 65536;
                this.c.d = 0;
                this.c.b = 0;
                this.c.e = 65536;
            }
            if (!this.a && this.c.b < 0) {
                this.c.a = -65536;
                this.c.d = 0;
                this.c.b = 0;
                this.c.e = 65536;
            }
            this.h();
        }
    }

    public final void f() {
        if (this.s == 0) {
            this.s = t;
        }
    }

    public final void d() {
        super.d();
        this.r -= p.a;
        if (this.r < 0) {
            this.r = 0;
        }
        if (this.s > 0) {
            this.s -= p.a;
            if (this.s <= u && g.a[1] == 1) {
                c c2 = (c)this.a().a(this.a);
                ((c)this.a().a(this.a)).h = 0.0f;
                c2.i = 0.0f;
                c2.a = this.a * this.c.a >> 12;
                c2.b = this.a * this.c.d >> 12;
                c2.d = false;
                c2.c = true;
                c2.a = true;
                c2.b = true;
                c2.j = 0.0f;
                c2.k = 0.0f;
                g.a[1] = 0;
                this.r = 500;
                this.a(k.a);
                k.a.a(0x780000, 0);
                int n2 = d.g;
                int n3 = d.h;
                c2.c.c = n2;
                c2.c.f = n3;
                n.e.a(10, n2, n3, 800, 200, this.c.a, this.c.d, 30, 800, 200);
            }
            if (this.s <= 0) {
                this.s = 0;
                for (int i2 = 0; i2 < 3; ++i2) {
                    q q2 = (q)n.a[i2 + 9];
                    q q3 = (q)n.a[i2];
                    for (int i3 = 0; i3 < q2.c.length; ++i3) {
                        q2.c[i3] = q3.c[i3];
                        q2.d[i3] = q3.d[i3];
                    }
                }
            }
        }
    }

    public final void a(Graphics graphics, DirectGraphics directGraphics, d d2) {
        int n2;
        if (this.s > 0) {
            int n3;
            int n4 = t - this.s;
            int n5 = 0;
            n2 = 0;
            for (n3 = 0; n3 < 3 && n4 >= n5 + a[n3]; ++n3) {
                n5 += a[n3];
                ++n2;
            }
            n3 = n2;
            float f2 = (float)(n4 -= n5) / (float)a[n3];
            float f3 = 1.0f - f2;
            for (n2 = 0; n2 < 3; ++n2) {
                q q2 = (q)n.a[n3 * 3 + n2];
                q q3 = (q)n.a[n2 + 9];
                q q4 = n3 == 2 ? (q)n.a[n2] : (q)n.a[(n3 + 1) * 3 + n2];
                for (int i2 = 0; i2 < q3.c.length; ++i2) {
                    q3.c[i2] = (int)((float)q2.c[i2] * f3 + (float)q4.c[i2] * f2);
                    q3.d[i2] = (int)((float)q2.d[i2] * f3 + (float)q4.d[i2] * f2);
                }
            }
        }
        this.a(k.a);
        d.a(d2, k.a, d.a);
        int n6 = d.a.c >> 16;
        int n7 = d.a.f >> 16;
        for (n2 = 0; n2 < 3; ++n2) {
            k k2 = n.a[n2 + 9];
            n.a[n2 + 9].c.c = this.c.c;
            k2.c.f = this.c.f;
            k2.c.a = this.c.a;
            k2.c.d = this.c.d;
            k2.c.b = this.c.b;
            k2.c.e = this.c.e;
            k2.h();
            k2.a(graphics, directGraphics, d2);
        }
        p.a(n6, n7, 48);
    }
}

