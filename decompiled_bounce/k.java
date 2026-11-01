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
public class k {
    public static final d a = new d();
    public final d b;
    public final d c;
    private d h;
    public final d d;
    public final d e;
    private k c;
    private k d;
    private k e;
    private k f;
    private short a;
    private short b;
    public int e;
    public int f;
    public int g;
    public int h;
    public int i;
    public int j;
    public int k;
    public int l;
    public byte b;
    public int m;
    public boolean e = true;
    private boolean a = true;
    public boolean f = true;
    public boolean g = true;
    private short c;
    protected byte c = 1;
    private static short d;
    private static int a;
    public static final d f;
    private static d i;
    public static final d g;
    private static d j;
    private static d k;
    public static k a;
    public static int n;
    public static int o;
    public static int p;
    public static int q;
    private static int b;
    private static k[] a;
    protected static k b;

    public k() {
        short s2 = d;
        d = (short)(s2 + 1);
        this.c = s2;
        this.b = new d();
        this.c = new d();
        this.h = new d();
        this.d = new d();
        this.e = new d();
    }

    public final byte a() {
        return this.c;
    }

    public void d() {
        if (this.e) {
            if (this.c != null) {
                d.a(this.c.b, this.c, this.b);
            } else {
                this.b.a(this.c);
            }
            this.b.c(this.d);
            this.e = false;
        }
    }

    public final void h() {
        for (k k2 = this; k2 != null; k2 = k2.a(this)) {
            k2.e = true;
            k2.f = true;
            k2.g = true;
        }
    }

    public final void i() {
        while (k2 != null) {
            k2.a = true;
            k k2 = k2.c;
        }
    }

    public void a(k k2) {
    }

    public void a(Graphics graphics, DirectGraphics directGraphics, d d2) {
    }

    public final void b(k k2) {
        if (k2.a(this)) {
            throw new IllegalArgumentException();
        }
        if (k2 == this) {
            throw new IllegalArgumentException();
        }
        this.j();
        this.e = k2.d;
        if (this.e != null) {
            this.e.f = this;
        }
        this.c = k2;
        this.c.d = this;
    }

    public final void j() {
        if (this.c != null) {
            if (this.c.d == this) {
                this.c.d = this.e;
                if (this.e != null) {
                    this.e.f = null;
                }
            } else {
                if (this.f != null) {
                    this.f.e = this.e;
                }
                if (this.e != null) {
                    this.e.f = this.f;
                }
            }
            this.c = null;
            this.e = null;
            this.f = null;
        }
    }

    public final void a(d d2) {
        if (this.f) {
            this.k();
        }
        d2.a(this.h);
    }

    public final void k() {
        this.h.a(this.c);
        k k2 = this.c;
        while (k2 != null) {
            d.a(k2.c, this.h, this.h);
            k2 = k2.c;
        }
        this.h.c(this.e);
        this.f = false;
    }

    public final boolean a(k k2) {
        k k3 = k3.c;
        while (k3 != null) {
            if (k2 == k3) {
                return true;
            }
            k3 = k3.c;
        }
        return false;
    }

    public final k a(k k2) {
        if (k2 == null) {
            return null;
        }
        if (k3.d != null) {
            return k3.d;
        }
        if (k3 == k2) {
            return null;
        }
        if (k3.e != null) {
            return k3.e;
        }
        k k3 = k3.c;
        while (k3 != null && k3 != k2) {
            if (k3.e != null) {
                return k3.e;
            }
            k3 = k3.c;
        }
        return null;
    }

    public final k b(k k2) {
        if (k2 == null) {
            return null;
        }
        if (k3 == k2) {
            return null;
        }
        if (k3.e != null) {
            return k3.e;
        }
        k k3 = k3.c;
        while (k3 != null && k3 != k2) {
            if (k3.e != null) {
                return k3.e;
            }
            k3 = k3.c;
        }
        return null;
    }

    public final short a() {
        return this.c;
    }

    public final void a(short s2) {
        this.c = s2;
    }

    public final k a(short s2) {
        k k2;
        for (k2 = this; k2 != null && k2.c != s2; k2 = k2.a(this)) {
        }
        return k2;
    }

    public final k a() {
        while (k2.c != null) {
            k k2 = k2.c;
        }
        return k2;
    }

    public void a() {
        this.e = Integer.MAX_VALUE;
        this.f = Integer.MAX_VALUE;
        this.g = Integer.MIN_VALUE;
        this.h = Integer.MIN_VALUE;
    }

    public final void l() {
        if (this.a) {
            this.i = this.e;
            this.j = this.f;
            this.k = this.g;
            this.l = this.h;
            k k2 = this;
            k2 = k2.d;
            while (k2 != null) {
                k2.l();
                k2.c.a(k2.i, k2.j);
                if (d.g < this.i) {
                    this.i = d.g;
                }
                if (d.h < this.j) {
                    this.j = d.h;
                }
                if (d.g > this.k) {
                    this.k = d.g;
                }
                if (d.h > this.l) {
                    this.l = d.h;
                }
                k2.c.a(k2.i, k2.l);
                if (d.g < this.i) {
                    this.i = d.g;
                }
                if (d.h < this.j) {
                    this.j = d.h;
                }
                if (d.g > this.k) {
                    this.k = d.g;
                }
                if (d.h > this.l) {
                    this.l = d.h;
                }
                k2.c.a(k2.k, k2.l);
                if (d.g < this.i) {
                    this.i = d.g;
                }
                if (d.h < this.j) {
                    this.j = d.h;
                }
                if (d.g > this.k) {
                    this.k = d.g;
                }
                if (d.h > this.l) {
                    this.l = d.h;
                }
                k2.c.a(k2.k, k2.j);
                if (d.g < this.i) {
                    this.i = d.g;
                }
                if (d.h < this.j) {
                    this.j = d.h;
                }
                if (d.g > this.k) {
                    this.k = d.g;
                }
                if (d.h > this.l) {
                    this.l = d.h;
                }
                k2 = k2.e;
            }
            this.a = false;
        }
    }

    public int a(byte[] byArray, int n2) {
        this.a = k.a(byArray, n2);
        this.b = k.a(byArray, n2 += 2);
        byte by = byArray[n2 += 2];
        ++n2;
        if ((by & 7) == 7) {
            this.c.a = k.b(byArray, n2);
            this.c.b = k.b(byArray, n2 += 4);
            this.c.c = k.b(byArray, n2 += 4);
            this.c.d = k.b(byArray, n2 += 4);
            this.c.e = k.b(byArray, n2 += 4);
            this.c.f = k.b(byArray, n2 += 4);
            n2 += 4;
        } else {
            if ((by & 1) > 0) {
                this.c.c = k.a(byArray, n2) << 16;
                this.c.f = k.a(byArray, n2 += 2) << 16;
                n2 += 2;
            }
            if ((by & 2) > 0) {
                this.c.a((float)k.b(byArray, n2) / 65536.0f);
                n2 += 4;
            }
            if ((by & 4) > 0) {
                this.c.a = k.b(byArray, n2);
                this.c.e = k.b(byArray, n2 += 4);
                n2 += 4;
            }
        }
        this.m = k.b(byArray, n2);
        this.b = (byte)((this.m & 0x1F) - 16);
        this.b.a = this.c.a;
        this.b.b = this.c.b;
        this.b.c = this.c.c;
        this.b.d = this.c.d;
        this.b.e = this.c.e;
        this.b.f = this.c.f;
        return n2 + 4;
    }

    public static void a(k[] kArray) {
        int n2;
        for (n2 = 0; n2 < kArray.length; ++n2) {
            k k2;
            k k3 = kArray[n2];
            if (k3.a > -1) {
                k3.c = k2 = kArray[k3.a];
                if (k2.d == null) {
                    k2.d = k3;
                }
            }
            if (k3.b <= -1) continue;
            k3.f = k2 = kArray[k3.b];
            k2.e = k3;
        }
        for (n2 = 0; n2 < kArray.length; ++n2) {
            kArray[n2].e = true;
            kArray[n2].a = true;
            kArray[n2].f = true;
            kArray[n2].k();
            kArray[n2].a(a);
            kArray[n2].b.a(a);
            kArray[n2].b.c(kArray[n2].d);
        }
    }

    public static final short a(byte[] byArray, int n2) {
        int n3 = 0 | byArray[n2];
        n3 <<= 8;
        return (short)(n3 |= byArray[n2 + 1] & 0xFF);
    }

    public static final int b(byte[] byArray, int n2) {
        long l2 = 0L | (long)byArray[n2];
        l2 <<= 8;
        l2 |= (long)(byArray[n2 + 1] & 0xFF);
        l2 <<= 8;
        l2 |= (long)(byArray[n2 + 2] & 0xFF);
        l2 <<= 8;
        return (int)(l2 |= (long)(byArray[n2 + 3] & 0xFF));
    }

    public static int a(int[] nArray, int n2, int n3, byte[] byArray, int n4, int n5) {
        int n6 = 0;
        int n7 = 0;
        int n8 = 0;
        int n9 = 1 << n5 - 1;
        int n10 = (1 << n5) - 1;
        while (n8 < n2) {
            n6 |= byArray[n4++] << n7;
            n6 &= (1 << (n7 += 8)) - 1;
            while (n7 >= n5) {
                int n11 = n6 & n10;
                if ((n11 & n9) > 0) {
                    n11 |= ~n10;
                }
                if (n8 < n2) {
                    nArray[n8++] = n11 + n3 << 16;
                }
                n7 -= n5;
                n6 >>>= n5;
            }
        }
        return n4;
    }

    public static int a(short[] sArray, int n2, int n3, int n4, byte[] byArray, int n5, int n6) {
        n4 = 0;
        int n7 = 0;
        int n8 = 0;
        int n9 = 1 << n6 - 1;
        int n10 = (1 << n6) - 1;
        while (n8 < n2) {
            n4 |= byArray[n5++] << n7;
            n4 &= (1 << (n7 += 8)) - 1;
            while (n7 >= n6) {
                int n11 = n4 & n10;
                if ((n11 & n9) > 0) {
                    n11 |= ~n10;
                }
                if (n8 < n2) {
                    sArray[n8++] = (short)(n11 + n3);
                }
                n7 -= n6;
                n4 >>>= n6;
            }
        }
        return n5;
    }

    public static final boolean a(int n2, int n3, int n4, int n5, int n6, int n7, int n8, int n9) {
        long l2;
        long l3;
        long l4 = n8 - n6;
        long l5 = n9 - n7;
        long l6 = n4 - n2;
        long l7 = n5 - n3;
        long l8 = n2 + n4 - n6 - n8;
        long l9 = n3 + n5 - n7 - n9;
        if ((l8 > 0L ? l8 : -l8) > l4 + (l3 = l6 > 0L ? l6 : -l6)) {
            return false;
        }
        if ((l9 > 0L ? l9 : -l9) > l5 + (l2 = l7 > 0L ? l7 : -l7)) {
            return false;
        }
        long l10 = l8 * l7 - l9 * l6;
        return (l10 > 0L ? l10 : -l10) <= l4 * l2 + l5 * l3;
    }

    public static final boolean b(int n2, int n3, int n4, int n5, int n6, int n7, int n8, int n9) {
        if (n4 < n6 || n2 > n8) {
            return false;
        }
        return n5 >= n7 && n3 <= n9;
    }

    public static boolean a(int n2, int n3, int n4, int n5, int n6, int n7) {
        if (n2 < n4 || n2 > n6) {
            return false;
        }
        return n3 >= n5 && n3 <= n7;
    }

    public static void m() {
        n = 0;
        o = 0;
        k.b(true);
    }

    public static void b(boolean bl) {
        int n2;
        int n3 = p.a;
        int n4 = 0;
        int n5 = 0;
        a.a(a);
        if (n3 != 0) {
            n4 = (k.a.c - k.a.b.c) / n3;
            n5 = (k.a.f - k.a.b.f) / n3;
        }
        n4 <<= 7;
        n5 = -n5 << 7;
        int n6 = k.a.c;
        int n7 = k.a.f;
        int n8 = -p.c / 5 << 16;
        g.c(d.a);
        d.a.b(0, n8);
        n8 = d.g;
        int n9 = d.h;
        d.a.b(n4, n5);
        n4 = n6 + n8 + n4 + d.g;
        n5 = n7 + n9 + n5 + d.h;
        if (bl) {
            k.f.c = n4;
            k.f.f = n5;
            return;
        }
        int n10 = n4 - k.f.c;
        n4 = n5 - k.f.f;
        if (Math.abs(n10) < 327680) {
            n2 = 0;
        }
        if (Math.abs(n4) < 327680) {
            n4 = 0;
        }
        a += n3;
        while (a >= 15) {
            k.f.c += n * 15;
            k.f.f += o * 15;
            n += 15 * p * (n2 >> 6) >> 14;
            o += 15 * p * (n4 >> 6) >> 14;
            n -= 15 * q * n >> 14;
            o -= 15 * q * o >> 14;
            a -= 15;
        }
    }

    public static void a(k kArray, Graphics graphics, DirectGraphics directGraphics) {
        int n2;
        f.c(i);
        d.a(g, i, j);
        k.j.c >>= 16;
        k.j.c <<= 16;
        k.j.f >>= 16;
        k.j.f <<= 16;
        j.c(k);
        k.a(0, 0);
        int n3 = d.g;
        int n4 = d.h;
        k.a(p.b << 16, p.c << 16);
        int n5 = d.g;
        int n6 = d.h;
        if (n3 > n5) {
            n5 = n3;
            n3 = d.g;
        }
        if (n4 > n6) {
            n6 = n4;
            n4 = d.h;
        }
        b = 0;
        Object object = kArray;
        while (object != null) {
            ((k)object).a(a);
            a.a(((k)object).i, ((k)object).j);
            int n7 = d.g;
            int n8 = d.h;
            int n9 = d.g;
            int n10 = d.h;
            a.a(((k)object).i, ((k)object).l);
            if (d.g < n7) {
                n7 = d.g;
            }
            if (d.h < n8) {
                n8 = d.h;
            }
            if (d.g > n9) {
                n9 = d.g;
            }
            if (d.h > n10) {
                n10 = d.h;
            }
            a.a(((k)object).k, ((k)object).l);
            if (d.g < n7) {
                n7 = d.g;
            }
            if (d.h < n8) {
                n8 = d.h;
            }
            if (d.g > n9) {
                n9 = d.g;
            }
            if (d.h > n10) {
                n10 = d.h;
            }
            a.a(((k)object).k, ((k)object).j);
            if (d.g < n7) {
                n7 = d.g;
            }
            if (d.h < n8) {
                n8 = d.h;
            }
            if (d.g > n9) {
                n9 = d.g;
            }
            if (d.h > n10) {
                n10 = d.h;
            }
            k[] kArray2 = object;
            boolean bl = ((k)object).c == 11;
            if (bl || k.b(n7, n8, n9, n10, n3, n4, n5, n6)) {
                if ((((k)object).m & 0x80) > 0) {
                    object = ((k)object).a((k)kArray);
                    continue;
                }
                k.a[k.b] = object;
                ++b;
                object = ((k)object).a((k)kArray);
                continue;
            }
            object = ((k)object).b((k)kArray);
        }
        n3 = b;
        kArray = a;
        if (a != null && n3 > 1) {
            if (n3 == 2) {
                if (kArray[0].b < kArray[1].b) {
                    k k2 = kArray[0];
                    kArray[0] = kArray[1];
                    kArray[1] = k2;
                }
            } else {
                k.a(kArray, 0, n3 - 1);
            }
        }
        for (n2 = 0; n2 < b; ++n2) {
            a[n2].a(graphics, directGraphics, j);
        }
        b = 0;
        for (n2 = 0; n2 < a.length; ++n2) {
            k.a[n2] = null;
        }
    }

    private static final void a(k[] kArray, int n2, int n3) {
        while (n2 < n3) {
            int n4 = n3;
            int n5 = n2;
            k[] kArray2 = kArray;
            k k2 = kArray[n4];
            int n6 = n5 - 1;
            while (n5 < n4) {
                if (k2.b < kArray2[n5].b) {
                    k k3 = kArray2[++n6];
                    kArray2[n6] = kArray2[n5];
                    kArray2[n5] = k3;
                }
                ++n5;
            }
            kArray2[n4] = kArray2[n6 + 1];
            kArray2[n6 + 1] = k2;
            int n7 = n6 + 1;
            k.a(kArray, n2, n7 - 1);
            n2 = n7 + 1;
        }
    }

    static {
        a = 0;
        f = new d();
        i = new d();
        g = new d();
        j = new d();
        k = new d();
        k.g.a = 43266;
        k.g.e = -43266;
        if (p.b < 200) {
            k.g.a = 22306;
            k.g.e = -22306;
        }
        b = 0;
        a = new k[60];
        b = new k();
    }
}

