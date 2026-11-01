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
public final class c
extends k {
    public int a;
    private int r;
    private static float l = 0.0f;
    private static float m = -400.0f;
    private static float[] b = new float[]{0.6f, 0.3f, 0.6f};
    private static float[] c = new float[]{0.1f, 0.1f, 0.1f};
    private static float[] d = new float[]{(float)Math.cos(0.9599310755729675), (float)Math.cos(0.9599310755729675), (float)Math.cos(0.9599310755729675)};
    public static final int[] a = new int[]{20, 20, 20};
    public static final float[] a = new float[]{1.0f, 1.4f, 0.5f};
    private static float[] e = new float[]{500.0f, 350.0f, 312.5f};
    private static float[] f = new float[]{280.0f, 350.0f, 150.0f};
    private static float[] g = new float[]{1.5f, 2.5f, 1.2f};
    private static float[] h = new float[]{288.0f, 255.0f, 162.5f};
    public static int[] b = new int[3];
    private static int s;
    public static int b;
    public static int c;
    private static int[] c;
    private static short[] a;
    public float a;
    public float b;
    public float c;
    public float d;
    public float e;
    public float f;
    public float g;
    public boolean a;
    public boolean b = true;
    public boolean c = false;
    private int[] d;
    private int[] e;
    private int[] f;
    private int[] g;
    private int[] h;
    private int[] i;
    public float h;
    public float i;
    public float j;
    public float k;
    private float n;
    private int t;
    private final int[] j = new int[16];
    private final int[] k = new int[16];
    private final int[] l = new int[16];
    private final int[] m = new int[16];
    private final int[] n = new int[16];
    private final int[] o;
    private final boolean[] a = true;
    private float o = 0.5f;
    private int u = 15539236;
    private int v = 0xFFFFFF;
    private int w = 11473152;
    private static int[] p;
    public int d;
    private float p;
    private float q;
    private float r = 0;
    public boolean d = false;
    private boolean h = false;
    private static int x;
    private static int y;
    private static int z;
    private static boolean i;
    private static float s;

    public c(boolean bl) {
        this.a = new boolean[16];
        this.c = (float[])4;
        this.h = bl;
        if (bl) {
            this.d = new int[4];
            this.e = new int[4];
            this.f = new int[4];
            this.g = new int[4];
            this.h = new int[4];
            this.i = new int[4];
            this.n();
        }
    }

    private void n() {
        for (int i2 = 0; i2 < 4; ++i2) {
            this.d[i2] = 0;
            this.e[i2] = 0;
            this.f[i2] = 0;
            this.g[i2] = 0;
        }
    }

    public final void b() {
        float f2 = 0.0f;
        if (n.d && this.d == 0) {
            f2 = 50.0f;
        }
        if (this.d) {
            if (this.a > -f[this.d] - f2) {
                this.d -= e[this.d] + f2;
                return;
            }
        } else if (this.a > -f[this.d] - f2) {
            this.d -= (e[this.d] + f2) / g[this.d];
        }
    }

    public final void c() {
        float f2 = 0.0f;
        if (n.d && this.d == 0) {
            f2 = 50.0f;
        }
        if (this.d) {
            if (this.a < f[this.d] + f2) {
                this.d += e[this.d] + f2;
                return;
            }
        } else if (this.a < f[this.d] + f2) {
            this.d += (e[this.d] + f2) / g[this.d];
        }
    }

    public final void a(boolean bl) {
        if (this.r > d[this.d] && this.d) {
            this.g = bl ? (this.g += h[this.d] / 2.0f) : (this.g += h[this.d]);
            this.d = false;
            if (this.h) {
                if (bl) {
                    this.b(0, -27);
                    this.b(1, -27);
                    return;
                }
                this.b(0, -53);
                this.b(1, -53);
            }
        }
    }

    private void b(int n2, int n3) {
        if (n3 > 0) {
            if (n3 > 70) {
                n3 = 70;
            }
        } else if (n3 < 0 && n3 < -70) {
            n3 = -70;
        }
        int n4 = n2;
        this.d[n4] = this.d[n4] + n3;
    }

    public final void a() {
        this.e = (float[])(-a[this.d] << 16);
        this.g = (float[])(a[this.d] << 16);
        this.f = (float[])(-a[this.d] << 16);
        this.h = (float[])(a[this.d] << 16);
    }

    public final void d() {
        float f2;
        this.h();
        super.d();
        this.f = (float[])true;
        if (!this.a) {
            return;
        }
        this.d += 0.0f * a[this.d];
        this.e += m * a[this.d];
        if (this.h) {
            if (!this.c) {
                float f3 = this.a - this.h;
                f2 = this.b - this.i;
                if (f3 > 0.0f) {
                    this.b(3, ((int)f3 >> 2 << 1) / 3);
                } else {
                    this.b(2, (-((int)f3) >> 2 << 1) / 3);
                }
                if (f2 > 0.0f) {
                    this.b(1, ((int)f2 >> 2 << 1) / 3);
                } else {
                    this.b(0, (-((int)f2) >> 2 << 1) / 3);
                }
            }
            this.c = false;
        }
        c c2 = this;
        f2 = (float)p.a * 0.001f;
        if (c2.h) {
            c2.h = c2.a;
            c2.i = c2.b;
        }
        float f4 = 1.0f / a[c2.d];
        c2.a += c2.d * f4 * f2;
        c2.b += c2.e * f4 * f2;
        c2.a += c2.f * f4;
        c2.b += c2.g * f4;
        c2.c.c += (int)(c2.a * f2 * 65536.0f);
        c2.c.f += (int)(c2.b * f2 * 65536.0f);
        c2.d = 0.0f;
        c2.e = 0.0f;
        c2.f = 0.0f;
        c2.g = 0.0f;
        f4 = c2.j * c2.q + c2.k * c2.r;
        float f5 = f4 * c2.q;
        f4 *= c2.r;
        if (c2.q * c2.j + c2.r * c2.k >= 0.0f) {
            f5 = c2.j - f5;
            f4 = c2.k - f4;
        } else {
            f5 = c2.j + f5;
            f4 = c2.k + f4;
        }
        float f6 = (float)Math.sqrt(f5 * f5 + f4 * f4);
        f2 = f6 / (float)a[c2.d] * f2;
        if (c2.q * f4 - f5 * c2.r > 0.0f) {
            f2 = -f2;
        }
        c2.n += f2;
        this.p += (float)p.a / 1000.0f;
        if (this.p > 0.25f) {
            this.d = false;
        }
        this.c = (float)Math.sqrt(this.a * this.a + this.b * this.b);
        if (this.c > 999.0f) {
            float f7 = 999.0f / this.c;
            this.a *= f7;
            this.b *= f7;
            this.c = 999.0f;
        }
        if (this.h) {
            for (int i2 = 0; i2 < 4; ++i2) {
                int n2 = i2;
                this.g[n2] = this.g[n2] + this.d[i2];
                this.d[i2] = 0;
                if (this.g[i2] > 70) {
                    this.g[i2] = 70;
                } else if (this.g[i2] < -70) {
                    this.g[i2] = -70;
                }
                int n3 = i2;
                this.e[n3] = this.e[n3] + this.g[i2] * p.a;
                this.f[i2] = this.e[i2] * k.g.a / 65536;
                int n4 = i2;
                this.g[n4] = this.g[n4] + (p.a * 7 * -this.e[i2] >> 16);
                int n5 = i2;
                this.g[n5] = this.g[n5] - (p.a * 120 * this.g[i2] >> 16);
            }
        }
        if (this.h) {
            float f8 = 40.0f;
            if (s > 0) {
                b = 3000;
                if ((s -= p.a) <= 0) {
                    if (c >= 2 && c <= 8) {
                        int n6 = Math.abs(n.a.nextInt() % 3) == 0 ? 3 : Math.abs(n.a.nextInt() % 6) + 3;
                        s = 1500;
                        c = n6;
                        if (c == 3) {
                            s = 600;
                        }
                    } else {
                        s = 0;
                        c = 0;
                    }
                }
            } else if (g.a[0] == 0 && g.a[1] == 0 && Math.abs(this.a) < f8 && Math.abs(this.b) < f8 && (b -= p.a) <= 0) {
                s = 600;
                c = 2;
                b = 3000;
            }
            if (Math.abs(this.a) >= f8 || Math.abs(this.b) >= f8) {
                if (c != 1) {
                    c = 0;
                }
                b = 3000;
            }
        }
        if (this.a != 0) {
            int n7 = this.a >>> 24;
            int n8 = p.a / 2;
            if (n8 < 1) {
                n8 = 1;
            }
            if ((n7 -= n8) < 0) {
                n7 = 0;
            }
            this.a = n7 << 24;
        }
    }

    public final void a(int n2, int n3) {
        this.g();
        this.c.c = n2;
        this.c.f = n3;
        this.b.c = n2;
        this.b.f = n3;
        this.c = true;
        if (this.h) {
            this.h = 0.0f;
            this.i = 0.0f;
            for (n2 = 0; n2 < 4; ++n2) {
                this.d[n2] = 0;
                this.e[n2] = 0;
                this.f[n2] = 0;
                this.g[n2] = 0;
            }
        }
        this.f = (float[])true;
    }

    public final void a(k k2) {
        int n2 = 1;
        while (++n2 < 10) {
            k k3 = k2;
            c c2 = this;
            this.t = 0;
            int n3 = a[c2.d] << 16;
            int n4 = a[c2.d] * a[c2.d] << 16;
            int n5 = c2.c.c - c2.b.c;
            int n6 = c2.c.f - c2.b.f;
            k k4 = k3;
            block10: while (k4 != null) {
                int n7;
                int n8;
                k4.d.a(c2.b.c, c2.b.f);
                int n9 = d.g;
                int n10 = d.h;
                if (k4.f) {
                    k4.k();
                }
                k4.e.a(c2.c.c, c2.c.f);
                int n11 = d.g;
                int n12 = d.h;
                int n13 = n9;
                int n14 = n11;
                if (n13 > n14) {
                    n13 = n11;
                    n14 = n9;
                }
                if ((n8 = n10) > (n7 = n12)) {
                    n8 = n12;
                    n7 = n10;
                }
                if (!c.b(k4.i, k4.j, k4.k, k4.l, n13 -= n3, n8 -= n3, n14 += n3, n7 += n3)) {
                    k4 = k4.b(k3);
                    continue;
                }
                if ((k4.m & 0x20) > 0) {
                    k4 = k4.a(k3);
                    continue;
                }
                switch (k4.a()) {
                    case 6: {
                        k k5 = (i)k4;
                        int n15 = ((i)k5).a << 16;
                        int n16 = ((i)k5).c << 16;
                        boolean bl = c.a(n15, n16, (((i)k5).b << 16) - n15, 0, n9, n10, n11, n12, 0);
                        int n17 = bl ? 1 : 0;
                        if (bl) {
                            ((i)k5).a(z, c2.b, a[c2.d], c2);
                        }
                        if (n12 - n3 < ((i)k5).b) {
                            ((i)k5).a(n11, n12 - n3, c2);
                        }
                        k4 = k4.a(k3);
                        continue block10;
                    }
                    case 8: {
                        k k5 = (e)k4;
                        int n18 = -4587520;
                        int n16 = 0x5F0000;
                        boolean bl = c.a(n18, n16, 0x8C0000, 0, n9, n10, n11, n12, 0);
                        int n17 = bl ? 1 : 0;
                        if (bl) {
                            c c3 = c2;
                            float f2 = c2.b;
                            k k6 = k5;
                            if (f2 < 0.0f) {
                                n12 = -((int)f2);
                                if (n12 < 100) {
                                    n12 = 100;
                                }
                                if (n12 > 1000) {
                                    n12 = 1000;
                                }
                                ((e)k6).b = (1100 - n12 >> 1) + 20;
                                ((e)k6).a = ((e)k6).b / p.b(((e)k6).a);
                                ((e)k6).a = c3;
                                ((e)k6).a = -f2 * (float)((e)k6).a * 2.0f / 100.0f;
                                f2 = 400.0f * (float)((e)k6).a * 2.0f / 100.0f;
                                if (((e)k6).a > f2) {
                                    ((e)k6).a = f2;
                                }
                                ((e)k6).a.a = false;
                                ((e)k6).a.a = 0.0f;
                                ((e)k6).a.b = 0.0f;
                                ((e)k6).a = false;
                                ((e)k6).b();
                                if (((e)k6).a.equals(n.a)) {
                                    g.a[1] = 2;
                                }
                            }
                        }
                        k4 = k4.a(k3);
                        continue block10;
                    }
                    case 7: {
                        k k5 = (l)k4;
                        boolean bl = c.a(n9, n10, n11, n12, ((l)k5).a, ((l)k5).c, ((l)k5).b, ((l)k5).d);
                        if (bl) {
                            ((l)k5).b();
                        }
                        k4 = k4.a(k3);
                        continue block10;
                    }
                    case 9: {
                        k k5 = (a)k4;
                        k k7 = k4.a(k3);
                        int n16 = n11 >> 16;
                        int n17 = n12 >> 16;
                        int n19 = n16 * n16 + n17 * n17;
                        if (n19 < 2025) {
                            k k8 = k5;
                            k8.a(k.a);
                            n10 = k.a.c;
                            n11 = k.a.f;
                            n.f.a(8, n10, n11, 540, 0, 540, 0);
                            if (k8.equals(n.a)) {
                                ++n.g;
                                ((a)k8).c.c = Integer.MAX_VALUE;
                                ((a)k8).c.f = Integer.MAX_VALUE;
                                ((a)k8).b.a(((a)k8).c);
                                ((a)k8).b.c(((a)k8).d);
                                ((a)k8).f = true;
                            } else {
                                k8.j();
                                ++n.g;
                            }
                        }
                        k4 = k7;
                        continue block10;
                    }
                    case 4: {
                        k4 = k4.a(k3);
                        continue block10;
                    }
                    case 10: {
                        int n20;
                        int n19;
                        int n17;
                        k k5 = (m)k4;
                        k k9 = k4.a(k3);
                        int n16 = 0;
                        if (((m)k5).a == 0 || ((m)k5).a == 2 && (((m)k5).d == 1 || ((m)k5).a)) {
                            n16 = c.a(n9, n10, n11, n12, ((m)k5).e, ((m)k5).f + (((m)k5).h - ((m)k5).f << 1) / 3, ((m)k5).g, ((m)k5).h) ? 1 : 0;
                        }
                        if (n16 != 0) {
                            ((m)k5).e();
                        } else {
                            boolean bl = c.a(n9, n10, n11, n12, ((m)k5).e, ((m)k5).f, ((m)k5).g, ((m)k5).h);
                            n17 = bl ? 1 : 0;
                            if (bl) {
                                ((m)k5).c();
                            }
                            if (((m)k5).a == 2) {
                                n19 = ((m)k5).a * 100 / ((m)k5).b;
                                n20 = (((m)k5).h - ((m)k5).f) * n19 / 100;
                                boolean bl2 = c.a(n9, n10, n11, n12, ((m)k5).e, ((m)k5).f, ((m)k5).g, ((m)k5).f + n20);
                                n17 = bl2 ? 1 : 0;
                                if (bl2) {
                                    ((m)k5).b();
                                }
                            }
                        }
                        k4 = k9;
                        continue block10;
                    }
                    case 2: {
                        k k5 = (q)k4;
                        boolean bl = false;
                        int n16 = 0;
                        int n17 = 0;
                        int n19 = 0;
                        int n20 = 0;
                        int n21 = 0;
                        int n22 = ((q)k5).a - 1;
                        for (int i2 = 0; i2 < n22; ++i2) {
                            long l2;
                            int n23;
                            int n24;
                            int n25 = ((q)k5).c[i2];
                            int n26 = ((q)k5).d[i2];
                            int n27 = ((q)k5).c[i2 + 1];
                            int n28 = ((q)k5).d[i2 + 1];
                            int n29 = n25;
                            int n30 = n27;
                            if (n29 > n30) {
                                n29 = n27;
                                n30 = n25;
                            }
                            if ((n24 = n26) > (n23 = n28)) {
                                n24 = n28;
                                n23 = n26;
                            }
                            if (!c.b(n29, n24, n30, n23, n13, n8, n14, n7)) continue;
                            if (!bl) {
                                bl = true;
                                n16 = n11 - n9;
                                n17 = n12 - n10;
                                l2 = (long)n16 * (long)n16 + (long)n17 * (long)n17;
                                n19 = (int)Math.sqrt(l2);
                                if (n19 != 0) {
                                    n20 = (int)(((long)n16 << 16) / (long)n19);
                                    n21 = (int)(((long)n17 << 16) / (long)n19);
                                }
                            }
                            n29 = n26 - n28;
                            n30 = -(n25 - n27);
                            l2 = (long)n29 * (long)n29 + (long)n30 * (long)n30;
                            int n31 = (int)Math.sqrt(l2);
                            int n32 = (int)(((long)n29 * (long)a[c2.d] << 16) / (long)n31);
                            n31 = (int)(((long)n30 * (long)a[c2.d] << 16) / (long)n31);
                            int n33 = n25 + n32;
                            int n34 = n26 + n31;
                            n32 = n27 + n32;
                            n31 = n28 + n31;
                            if ((long)n16 * (long)n29 + (long)n17 * (long)n30 >= 0L) continue;
                            if (c.a(n9, n10, n16, n17, n33, n34, n32, n31, n4)) {
                                c2.a((q)k5, z, n5, n6, n29, n30, i);
                            }
                            if (c.c(n9, n10, n20, n21, n19, n25, n26, a[c2.d])) {
                                n29 = x - n25;
                                n30 = y - n26;
                                c2.a((q)k5, z, n5, n6, n29, n30, i);
                            }
                            if (!c.c(n9, n10, n20, n21, n19, n27, n28, a[c2.d])) continue;
                            n29 = x - n27;
                            n30 = y - n28;
                            c2.a((q)k5, z, n5, n6, n29, n30, i);
                        }
                        k4 = k4.a(k3);
                        continue block10;
                    }
                }
                k4 = k4.a(k3);
            }
            if (!(c2.t != 0)) break;
            c2 = this;
            long l3 = 42949672960000L;
            long l4 = Long.MAX_VALUE;
            n6 = -1;
            for (int i3 = 0; i3 < c2.t; ++i3) {
                long l5 = c2.j[i3] - c2.b.c;
                long l6 = c2.k[i3] - c2.b.f;
                long l7 = l5 * l5 + l6 * l6;
                if (c2.a[i3]) {
                    l7 = -l7;
                }
                if (l7 > l3) {
                    System.out.println("Sanity check failed! Found collision is too far, distance: " + Math.sqrt(l7) / 65536.0);
                    continue;
                }
                if (l7 >= l4) continue;
                l4 = l7;
                n6 = i3;
            }
            if (n6 == -1) continue;
            float f3 = 1000.0f / (float)p.a;
            float f4 = (float)c2.j[n6] * s;
            float f5 = (float)c2.k[n6] * s;
            float f6 = (float)c2.l[n6] * s;
            float f7 = (float)c2.m[n6] * s;
            float f8 = 1.0f / (float)Math.sqrt(f6 * f6 + f7 * f7);
            float f9 = (float)c2.n[n6] * s;
            float f10 = (float)c2.o[n6] * s;
            float f11 = f9 * (f6 *= f8) + f10 * (f7 *= f8);
            float f12 = f11 * f6;
            float f13 = f11 * f7;
            if (f6 * f9 + f7 * f10 < 0.0f) {
                f12 = -f12;
                f13 = -f13;
            }
            float f14 = f9 * f3;
            float f15 = f10 * f3;
            float f16 = (float)(c2.c.c - c2.j[n6]) * s;
            float f17 = (float)(c2.c.f - c2.k[n6]) * s;
            float f18 = f4 + f9;
            float f19 = f5 + f10;
            float f20 = f6 * 0.01f;
            float f21 = f7 * 0.01f;
            f11 = f16 * f6 + f17 * f7;
            float f22 = f11 * f6;
            float f23 = f11 * f7;
            float f24 = f16 - f22 - f22 * c[c2.d];
            float f25 = f17 - f23 - f23 * c[c2.d];
            float f26 = f4 + f24 + f12 + f20;
            float f27 = f5 + f25 + f13 + f21;
            f11 = c2.a * f6 + c2.b * f7;
            f22 = f11 * f6;
            f23 = f11 * f7;
            f24 = c2.a - f22 - f22 * c[c2.d];
            f25 = c2.b - f23 - f23 * c[c2.d];
            f11 = f14 * f6 + f15 * f7;
            f22 = f11 * f6;
            f23 = f11 * f7;
            c2.a = f22 + f24;
            c2.b = f23 + f25;
            f24 = c2.a - f14;
            f25 = c2.b - f15;
            float f28 = (float)Math.sqrt(f24 * f24 + f25 * f25);
            float f29 = f28 != 0.0f ? f24 / f28 : 0.0f;
            f28 = f28 != 0.0f ? f25 / f28 : 0.0f;
            f11 = -(f6 * 0.0f + f7 * m);
            float f30 = b[c2.d] * a[c2.d] * f11;
            f30 = a[c2.d] * f3;
            f29 = f24 * f30;
            f28 = f25 * f30;
            float f31 = f29 * f30;
            float f32 = f28 * f30;
            if (f29 * f29 + f28 * f28 < f31 * f31 + f32 * f32) {
                c2.d -= f29;
                c2.e -= f28;
            } else {
                c2.d -= f31;
                c2.e -= f32;
            }
            c2.j = c2.j * (1.0f - c2.o) + f24 * c2.o;
            c2.k = c2.k * (1.0f - c2.o) + f25 * c2.o;
            c2.p = 0.0f;
            c2.d = true;
            c2.q = f6;
            c2.r = f7;
            c2.b.c = (int)(f18 * 65536.0f);
            c2.b.f = (int)(f19 * 65536.0f);
            c2.c.c = (int)(f26 * 65536.0f);
            c2.c.f = (int)(f27 * 65536.0f);
            c2.k();
            c2.b.c((d)c2.d);
            c2.t = 0;
        }
        this.p += (float)p.a / 1000.0f;
        if (this.p > 0.25f) {
            this.d = false;
        }
    }

    private void a(q q2, int n2, int n3, int n4, int n5, int n6, boolean bl) {
        if (n2 > 0) {
            k2.j[k2.t] = k2.b.c + (int)((long)n3 * (long)n2 >> 16);
            k2.k[k2.t] = k2.b.f + (int)((long)n4 * (long)n2 >> 16);
            k2.a[k2.t] = bl;
            q2.b.b(n5, n6);
            n3 = d.g;
            n4 = d.h;
            q2.a(k.a);
            k.a.b(n5, n6);
            n5 = d.g;
            n6 = d.h;
            k2.l[k2.t] = (int)((long)n3 * (long)(65536 - n2) + (long)n5 * (long)n2 >> 16);
            k2.m[k2.t] = (int)((long)n4 * (long)(65536 - n2) + (long)n6 * (long)n2 >> 16);
        } else {
            if (n2 < 0) {
                throw new IllegalStateException("t < 0, t: " + n2);
            }
            q2.b.a(x, y);
            k2.j[k2.t] = d.g;
            k2.k[k2.t] = d.h;
            k2.a[k2.t] = bl;
            q2.b.a(n5, n6);
            k2.l[k2.t] = d.g;
            k2.m[k2.t] = d.h;
        }
        float f2 = (float)p.a / 1000.0f;
        q2.b.a(x, y);
        n4 = d.g;
        n5 = d.h;
        q2.a(k.a);
        k.a.a(x, y);
        n6 = (int)(f2 * 65536.0f);
        k2.n[k2.t] = d.g - n4 + n6 * 0;
        k2.o[k2.t] = d.h - n5 + n6 * 0;
        ++k2.t;
        if (q2.a > -1) {
            k k2 = k2.a().a(q2.a);
            k2 = (g)k2;
            ((g)k2).a(2);
        }
    }

    private static boolean c(int n2, int n3, int n4, int n5, int n6, int n7, int n8, int n9) {
        long l2 = n2 - n7;
        long l3 = n3 - n8;
        long l4 = l2 * (long)n4 + l3 * (long)n5 >> 16;
        if (l4 >= 0L) {
            return false;
        }
        long l5 = (l2 * l2 + l3 * l3 >> 16) - (long)(n9 * n9 << 16);
        if (l5 <= 0L) {
            z = 0;
            long l6 = l2 * l2 + l3 * l3;
            int n10 = (int)Math.sqrt(l6);
            int n11 = 0;
            int n12 = 0;
            if (n10 != 0) {
                n11 = (int)((l2 * (long)n9 << 16) / (long)n10);
                n12 = (int)((l3 * (long)n9 << 16) / (long)n10);
            }
            x = n7 + n11;
            y = n8 + n12;
            i = true;
            return true;
        }
        long l7 = (l4 * l4 >> 16) - l5;
        if (l7 < 0L) {
            return false;
        }
        long l8 = l7 << 16;
        long l9 = -l4 - (long)((int)Math.sqrt(l8));
        if (l9 <= (long)n6) {
            z = (int)((l9 << 16) / (long)n6);
            x = (int)((long)n2 + (l9 * (long)n4 >> 16));
            y = (int)((long)n3 + (l9 * (long)n5 >> 16));
            i = false;
            return true;
        }
        return false;
    }

    private static boolean a(int n2, int n3, int n4, int n5, int n6, int n7, int n8, int n9, int n10) {
        long l2 = (long)n6 * (long)n5 >> 16;
        long l3 = (long)n7 * (long)n4 >> 16;
        long l4 = (long)n8 * (long)n5 >> 16;
        long l5 = (long)n9 * (long)n4 >> 16;
        long l6 = l2 - l3 - l4 + l5;
        if (l6 == 0L) {
            return false;
        }
        long l7 = (l2 - l3 + ((long)n4 * (long)n3 >> 16) - ((long)n5 * (long)n2 >> 16) << 16) / l6;
        if (l7 < 0L || l7 > 65536L) {
            return false;
        }
        long l8 = ((long)n6 * (long)(n9 - n3) + (long)n7 * (long)(n2 - n8) + (long)n8 * (long)n3 - (long)n9 * (long)n2) / l6;
        if (l8 >= 0L && l8 <= 65536L) {
            z = (int)l8;
            x = (int)((long)n2 + (l8 * (long)n4 >> 16));
            y = (int)((long)n3 + (l8 * (long)n5 >> 16));
            i = false;
            return true;
        }
        if (l8 < 0L) {
            long l9 = n2 - n6;
            long l10 = n8 - n6;
            long l11 = n3 - n7;
            long l12 = n9 - n7;
            long l13 = l9 * l10 + l11 * l12 >> 16;
            if (l13 <= 0L) {
                return false;
            }
            long l14 = l10 * l10 + l12 * l12 >> 16;
            if (l13 >= l14) {
                return false;
            }
            long l15 = (long)n6 + ((l13 = (l13 << 16) / l14) * l10 >> 16);
            long l16 = l15 - (long)n2;
            long l17 = (long)n7 + (l13 * l12 >> 16);
            long l18 = l17 - (long)n3;
            long l19 = l16 * l16 + l18 * l18 >> 16;
            if (l19 > (long)n10) {
                return false;
            }
            z = 0;
            x = (int)l15;
            y = (int)l17;
            i = true;
            return true;
        }
        return false;
    }

    public final void e() {
        this.h();
        super.d();
        this.n();
        int n2 = 3000 - n.c;
        if (n2 <= 1000) {
            this.c.f = n.d + (n.b[n2 / 5 % 360] << 14);
            return;
        }
        this.c.f = n.d + (n.b[200] << 14) - (n2 - 1000) * 22500;
    }

    public final void a(Graphics graphics, DirectGraphics directGraphics, d d2) {
        if (!this.b) {
            return;
        }
        this.a(k.a);
        d.a(d2, k.a, d.a);
        d.a.a(this.c.c, this.c.f);
        int n2 = d.a.c >> 16;
        int n3 = d.a.f >> 16;
        d.a.a(-(a[this.d] << 16), a[this.d] << 16);
        int n4 = d.g >> 16;
        int n5 = d.h >> 16;
        int n6 = n2 - n4;
        int n7 = n3 - n5;
        int n8 = n6 >> 2;
        int n9 = n2 - n8;
        int n10 = n3 - n8;
        if (this.h) {
            if (this.d == 0) {
                int n11;
                n6 = n2;
                n7 = n3;
                Graphics graphics2 = p.a();
                if (this.a != 0) {
                    n.a.setColor(255);
                    n.a.fillRect(0, 0, n.a.getWidth(), n.a.getHeight());
                    n2 = n.a.getWidth() >> 1;
                    n3 = n.a.getHeight() >> 1;
                    graphics = n.a;
                    p.a(graphics);
                }
                for (n5 = 0; n5 < 4; ++n5) {
                    this.h[n5] = 0;
                    for (n8 = 0; n8 < 4; ++n8) {
                        if (n5 == n8) continue;
                        int n12 = n5;
                        this.h[n12] = this.h[n12] - (this.f[n8] >> 1);
                    }
                }
                for (n5 = 0; n5 < 4; ++n5) {
                    this.i[n5] = 999;
                }
                this.a(n2, n3, b[this.d] + 2, b[this.d] + 2, -16777216, graphics, false, false);
                this.a(n2, n3, b[this.d], b[this.d], this.u, graphics, true, false);
                n5 = b[this.d] * 90 / 100;
                this.a(n2, n3 + 2, n5, b[this.d], this.w, graphics, false, false);
                this.a(n2 + 1, n3 - 1, n5, b[this.d], this.u, graphics, false, true);
                d.a.a(5.3f);
                n8 = this.i[1];
                n5 = n8 >> 1;
                n8 = this.i[1] - n5 + 1;
                n9 = n2 + (d.a.a * n8 >> 16);
                n8 = n3 + (d.a.d * n8 >> 16);
                block3: for (n10 = 0; n10 < 12; ++n10) {
                    int n13 = 31 + Math.abs(n10 % 2);
                    float f2 = this.n + (float)n10 * 0.8f;
                    int n14 = 100 - (n10 << 2);
                    int n15 = n13;
                    float f3 = f2;
                    int n16 = n14;
                    n14 = n3;
                    n11 = n2;
                    c c2 = this;
                    d.a.a(f3);
                    for (int i2 = 0; i2 < 4; ++i2) {
                        int n17 = c2.i[i2] >> 1;
                        n17 >>= 1;
                        n17 = n16 * (c2.i[i2] - n17) / 100;
                        int n18 = n11 + (d.a.a * n17 >> 16);
                        n17 = n14 + (d.a.d * n17 >> 16);
                        if (i2 == 0) {
                            if (n18 < n11 || n17 > n14) continue;
                            p.a(n18, n17 + (c2.f[1] + c2.h[1] >> 10), n15);
                            continue block3;
                        }
                        if (i2 == 1) {
                            if (n18 > n11 || n17 > n14) continue;
                            p.a(n18, n17 + (c2.f[1] + c2.h[1] >> 10), n15);
                            continue block3;
                        }
                        if (i2 == 2) {
                            if (n18 > n11 || n17 < n14) continue;
                            p.a(n18, n17 + (c2.f[1] + c2.h[1] >> 10), n15);
                            continue block3;
                        }
                        if (n18 < n11 || n17 < n14) continue;
                        p.a(n18, n17 + (c2.f[1] + c2.h[1] >> 10), n15);
                        continue block3;
                    }
                }
                p.a(n2, n3, 17);
                this.a(n9, n8, n5 >> 1, b[this.d], this.v, graphics, false, false);
                p.a(n2, n3, 28);
                if (this.a != 0) {
                    int n19;
                    p.a(graphics2);
                    n.a.getRGB(n.a, 0, n.a.getWidth(), 0, 0, n.a.getWidth(), n.a.getHeight());
                    n10 = 0;
                    for (n19 = 0; n19 < n.a.getHeight(); ++n19) {
                        for (n11 = 0; n11 < n.a.getWidth(); ++n11) {
                            if (n.a[n10] == -16776961) {
                                n.a[n10] = 0;
                            } else {
                                int n20 = n10;
                                n.a[n20] = n.a[n20] - this.a;
                            }
                            ++n10;
                        }
                    }
                    n19 = n6 - (n.a.getWidth() >> 1);
                    n11 = n7 - (n.a.getHeight() >> 1);
                    p.a().drawRGB(n.a, 0, n.a.getWidth(), n19, n11, n.a.getWidth(), n.a.getHeight(), true);
                }
                if (n.d && !n.b) {
                    n10 = p.a * p.a();
                    this.r += n10;
                    if (this.r > 150) {
                        int n21 = g.a[4] / 120;
                        n.d.b(n21, n.a.c.c, n.a.c.f, a[0] << 15, 0, 0, 0, 0, 1000, 166);
                        this.r = 0;
                    }
                }
            } else if (this.d == 1) {
                float f4 = 11.25f;
                for (n7 = (int)Math.toDegrees(this.n); n7 < 0; n7 += 360) {
                }
                while (n7 > 359) {
                    n7 -= 360;
                }
                n4 = 31 - (int)((float)n7 / f4);
                p.a(n2, n3, (int)a[n4]);
            } else if (this.d == 2) {
                n6 = b[this.d] << 1;
                graphics.setColor(0);
                graphics.fillArc(n4 - 2, n5 - 2, n6 + 4, n6 + 4, 0, 360);
                for (n7 = 0; n7 < 6; ++n7) {
                    graphics.setColor(p[n7]);
                    graphics.fillArc(n4, n5, n6, n6, n7 * 60 - (int)Math.toDegrees(this.n), 60);
                }
                graphics.setColor(p[0]);
                graphics.fillArc(n9, n10, n8 << 1, n8 << 1, 0, 360);
                p.a(n2, n3, 5);
            }
            if (c == 1 || g.a[0] == 3) {
                p.a(n2, n3, 20);
                return;
            }
            if (c == 2 || c == 3) {
                int n22 = 462;
                n7 = 2;
                if (c == 3) {
                    n22 = 447;
                    n7 = 4;
                }
                if ((n8 = s * (n5 = n7) / 600) > n5 - 1) {
                    n8 = n5 - 1;
                }
                n8 = n5 - 1 - n8;
                p.b(n2, n3, n22, n8);
                return;
            }
            if (c >= 4 && c <= 8) {
                p.b(n2, n3, c[c - 4], 0);
                return;
            }
            if (g.a[0] == 4) {
                p.b(n2, n3, 465, 0);
                return;
            }
        } else {
            graphics.setColor(this.u);
            graphics.fillArc(n4, n5, n6 << 1, n7 << 1, 0, 360);
        }
    }

    public final void f() {
        int n2;
        ++this.d;
        if (this.d > 2) {
            this.d = 0;
        }
        if (this.d > (n2 = n.a())) {
            this.d = 0;
        }
        this.a();
    }

    public final void g() {
        this.a = 0.0f;
        this.b = 0.0f;
        this.f = 0.0f;
        this.g = 0.0f;
        this.d = 0.0f;
        this.e = 0.0f;
        this.j = 0.0f;
        this.k = 0.0f;
    }

    private void a(int n2, int n3, int n4, int n5, int n6, Graphics graphics, boolean bl, boolean bl2) {
        graphics.setColor(n6);
        n6 = 2;
        for (int i2 = 0; i2 < 4; ++i2) {
            int n7 = n4 - n4 * (this.f[n6] + this.h[n6] >> 10) / n5;
            int n8 = n4 - n4 * (this.f[i2 >> 1] + this.h[i2 >> 1] >> 10) / n5;
            if (bl2) {
                ++n7;
            }
            if (bl) {
                this.i[i2] = n7;
                if (n8 < n7) {
                    this.i[i2] = n8;
                }
            }
            int n9 = n2 - n7;
            int n10 = n3 - n8;
            graphics.fillArc(n9, n10 + (this.f[1] + this.h[1] >> 10), n7 << 1, n8 << 1, i2 * 90, 90);
            if (i2 == 0) {
                ++n6;
            }
            if (i2 != 2) continue;
            --n6;
        }
    }

    static {
        c = new int[]{458, 460, 454, 452, 456};
        c.b[0] = a[0] * k.g.a / 65536 + 1;
        c.b[1] = a[1] * k.g.a / 65536 + 1;
        c.b[2] = a[2] * k.g.a / 65536 + 1;
        a = new short[]{211, 212, 223, 234, 237, 238, 239, 240, 241, 242, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 235, 236};
        p = new int[]{0xEDEDED, 25788, 0xEDEDED, 14752000, 0xEDEDED, 15261445};
        s = 1.5258789E-5f;
    }
}

