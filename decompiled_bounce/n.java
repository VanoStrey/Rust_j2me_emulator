/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  com.nokia.mid.ui.DirectGraphics
 *  com.nokia.mid.ui.DirectUtils
 *  javax.microedition.lcdui.Graphics
 *  javax.microedition.lcdui.Image
 */
import com.nokia.mid.ui.DirectGraphics;
import com.nokia.mid.ui.DirectUtils;
import java.util.Random;
import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.Image;

/*
 * Duplicate member names - consider using --renamedupmembers true
 */
public final class n {
    public static final short[] a = new short[]{49, 45, 44, 47, 50, 46, 48, 36, 86, 19, 18, 17, 4, 12, 13, 11, 14, 15, 85, 82, 37, 87, 38, 39, 89, 2, 3, 35, 16, 0, 1, 20, 24, 25, 32, 26, 27, 28, 29, 33, 30, 31, 21, 22, 34, 23, 88, 83, 84, 5, 6, 8, 7, 42, 9, 10, 41, 43, 56, 57, 58, 65, 59, 60, 61, 62, 63, 64, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 51, 52, 53, 54, 55, 81, 40, 90};
    private static boolean e;
    private h a;
    private final h b = new h();
    private int j = 1;
    private static byte a;
    private static final byte[] a;
    private static final byte[] b;
    private static boolean f;
    private static short[] c;
    public static Random a;
    public static int a;
    public static int b;
    private static int k;
    private static int l;
    private static int m;
    private static int n;
    public static int c;
    public static int d;
    private static int o;
    private static float[] a;
    private static int[] c;
    private static int[] d;
    private static int p;
    private static short[] d;
    private static short[] e;
    private static short[] f;
    private static short[] g;
    private static short[] h;
    private static short[] i;
    private static short[] j;
    private static short[] k;
    private static short[] l;
    private static short[] m;
    private static short[] n;
    private static short[] o;
    private static short[] p;
    private static short[] q;
    private static short[] r;
    private static short[] s;
    private static short[] t;
    private static short[] u;
    private static short[] v;
    private static int q;
    private static int r;
    private static int s;
    private static int t;
    private static int u;
    private static int v;
    private static int w;
    public static final int e;
    private static int x;
    private static int y;
    public static short[] b;
    public static boolean a;
    public static a a;
    private static String a;
    public static o a;
    public static o b;
    public static o c;
    public static o d;
    public static o e;
    public static o f;
    public static o g;
    public static o h;
    private static int[] e;
    private static int[] f;
    private static int[] g;
    private static int[] h;
    private static int[] i;
    private static int[] j;
    private static int[] k;
    private static int[] l;
    public static o i;
    private static int[] m;
    private static Image[] a;
    private static Image[] b;
    public static Image a;
    public static Graphics a;
    public static int[] a;
    public static Image b;
    public static Graphics b;
    public static int[] b;
    private static boolean g;
    private static int z;
    private static int[] n;
    private static String[] a;
    private static boolean h;
    private static boolean i;
    private static int A;
    private int B = 0;
    private static boolean j;
    private int C;
    private long a = null;
    private static final int[] o;
    private static final int[] p;
    private static final int[] q;
    private static final int[] r;
    private boolean k = false;
    public static boolean b;
    public static int f;
    private static int D;
    private static k[] b;
    public static k[] a;
    public static k a;
    private static int E;
    private static g[] a;
    public static c a;
    public static l a;
    public static boolean c;
    public static int g;
    public static int h;
    public static int i;
    private static int F;
    public static boolean d;
    private static int G;
    private static int[] s;
    private static int[] t;
    private static int[] u;
    private static int[] v;
    private static int[] w;
    private static short[] w;
    private static boolean l;
    private static boolean m;
    private static int H;
    private static int I;
    private boolean n = false;
    private boolean o = false;
    private boolean p = false;
    private int J = -1;
    private int K = -1;
    private boolean q = false;

    public static int a() {
        int n2 = 0;
        if (n.a(d[0]) || f == d[0] && g.a[2] > 0) {
            n2 = 1;
        }
        if (n.a(d[1]) || f == d[1] && g.a[2] > 0) {
            n2 = 2;
        }
        return n2;
    }

    private static boolean b(int n2) {
        return f[(n2 << 2) + 1] > 0;
    }

    public static boolean a(int n2) {
        return f[(n2 <<= 2) + 1] > 0 && f[n2 + 1] < 9999;
    }

    private static void a(int n2, short s2, short s3, short s4) {
        if (s2 > f[n2 <<= 2]) {
            n.f[n2] = s2;
        }
        if (s3 < f[n2 + 1]) {
            n.f[n2 + 1] = s3;
        }
        if (s4 > f[n2 + 2]) {
            n.f[n2 + 2] = s4;
        }
        if (s4 > f[n2 + 3]) {
            n.f[n2 + 3] = s4;
        }
    }

    private static void e(int n2) {
        if (f[(n2 << 2) + 1] == 0) {
            n.f[(n2 << 2) + 1] = 9999;
        }
    }

    private static void f(int n2) {
        if (f[(n2 <<= 2) + 1] == 0 || f[n2 + 1] == 9999) {
            n.f[n2 + 1] = 300;
        }
    }

    private static boolean c(int n2) {
        for (int i2 = 0; i2 < c.length; i2 += 2) {
            if (n2 != c[i2]) continue;
            return true;
        }
        return false;
    }

    private static int c() {
        int n2 = 0;
        for (int i2 = 0; i2 < 15; ++i2) {
            n2 += f[i2 << 2];
        }
        return n2;
    }

    private static boolean a() {
        for (int i2 = 0; i2 < 15; ++i2) {
            int n2 = i2 << 2;
            if (f[n2] < d[i2 * 3]) {
                return false;
            }
            if (f[n2 + 1] <= e[i2 * 3]) continue;
            return false;
        }
        return true;
    }

    private static final void a() {
        int n2 = 20520;
        int n3 = 0;
        int n4 = 0;
        while (n3 < 360) {
            int n5 = n4 / 57;
            n4 += (n2 -= n5) / 57;
            n.b[n3++] = (short)n5;
        }
    }

    private void g(int n2) {
        this.b.a();
        this.b.b(0);
        this.b.b(1);
        this.b.b = null;
        this.b.a = null;
        this.b.a = n2;
        this.b.a(9, -2);
        this.b.b(13, -2);
        switch (n2) {
            case 17: {
                this.b.d(37);
                this.b.a(9, -2);
                this.b.b(13, -2);
                this.b.a(5, 256);
                this.b.b(6, q);
                this.b.b(5, 128);
                this.b.a(18, 16742400);
                this.b.a("", -1, 1);
                this.b.a(0, r.a(49), 0, -2, true);
                this.b.a(1, r.a(45), 0, 9, false);
                if (n.b(1)) {
                    this.b.a(new f(r.a(17), -1, this.b, 18));
                    this.b.a(new f(r.a(36), -1, this.b, 20));
                } else {
                    this.b.a(new f(r.a(36), -1, this.b, 18));
                    k = 0;
                }
                this.b.a(new f(r.a(19), -1, this.b, 10));
                this.b.a(new f(r.a(18), -1, this.b, 22));
                this.b.a(o);
                break;
            }
            case 31: {
                this.b.d(36);
                this.b.a(9, -3);
                this.b.b(13, -2);
                this.b.b(6, r);
                this.b.b(7, s);
                this.b.b(15, 2);
                this.b.b(16, 2);
                this.b.b(12, p.c - t);
                this.b.b(11, p.b - (u << 1) + 4);
                this.b.b(9, u - 2);
                this.b.a(r.a(88), -1, 1);
                if (this.o) {
                    this.b.a(0, r.a(47), 0, 33, true);
                } else if (this.n) {
                    this.b.a(0, r.a(47), 0, 32, true);
                } else {
                    this.b.a(0, r.a(47), 0, 18, true);
                }
                String[] stringArray = new String[1];
                String[] stringArray2 = stringArray;
                stringArray[0] = "" + p;
                f f2 = new f(r.a(43, stringArray2), -1, this.b, -1);
                this.b.a(f2);
                stringArray2[0] = g + "/" + G;
                f2 = new f(stringArray2[0], 102, this.b, -1);
                if (e) {
                    f2.a(2, 16);
                    f2.a(stringArray2[0], 102);
                }
                this.b.a(f2);
                stringArray2[0] = n.b(a / 1000);
                f2 = new f(stringArray2[0], 103, this.b, -1);
                if (e) {
                    f2.a(2, 16);
                    f2.a(stringArray2[0], 103);
                }
                this.b.a(f2);
                if (this.q) {
                    f2 = new f(r.a(41), -1, this.b, -1);
                    f2.a(1, 8);
                    f2.a(r.a(41), -1);
                    this.b.a(f2);
                }
                if (n.a(13) && !this.n && !n.c(f)) {
                    f f3;
                    boolean bl = false;
                    if (this.J > -1) {
                        short s2 = c[this.J];
                        bl = true;
                        f3 = new f(r.a(8), s2, this.b, -1);
                        if (e) {
                            f3.a(2, 0);
                        } else {
                            f3.a(2, 16);
                        }
                        f3.a(3, 64);
                        f3.a(1, 8);
                        f3.a(r.a(8), (int)s2);
                        this.b.a(f3);
                    }
                    if (this.K > -1) {
                        short s3 = c[this.K];
                        bl = true;
                        f3 = new f(r.a(7), s3, this.b, -1);
                        if (e) {
                            f3.a(2, 0);
                        } else {
                            f3.a(2, 16);
                        }
                        f3.a(3, 64);
                        f3.a(1, 8);
                        f3.a(r.a(7), (int)s3);
                        this.b.a(f3);
                    }
                    if (!bl) {
                        f f4 = new f(r.a(42), -1, this.b, -1);
                        f4.a(1, 8);
                        f4.a(r.a(42), -1);
                        this.b.a(f4);
                    }
                }
                this.n = false;
                this.o = false;
                this.J = -1;
                this.K = -1;
                this.q = false;
                n.a(this.b, false);
                break;
            }
            case 19: {
                break;
            }
            case 10: {
                this.b.d(36);
                this.b.a(9, -3);
                this.b.b(13, -2);
                this.b.b(15, 2);
                this.b.b(16, 2);
                this.b.b(6, r);
                this.b.b(7, s);
                this.b.b(12, p.c - t);
                this.b.b(11, p.b - (u << 1) + 4);
                this.b.b(9, u - 2);
                this.b.a(3, 0);
                this.b.a(2, 32);
                this.b.b(14, p.a(-3) << 1);
                this.b.a(r.a(19), -1, 1);
                this.b.a(1, r.a(44), 0, 17, true);
                n2 = 0;
                for (int i2 = 0; i2 < 15; ++i2) {
                    if (f[(i2 << 2) + 3] <= 0) continue;
                    n2 = 1;
                }
                if (n2 != 0) {
                    String[] stringArray = new String[1];
                    for (n2 = 0; n2 < 15; ++n2) {
                        String string;
                        String string2 = "0";
                        if (f[(n2 << 2) + 3] > 0) {
                            string2 = "" + f[(n2 << 2) + 3];
                        }
                        if (n.c(n2)) {
                            stringArray[0] = n.a(n2);
                            string = r.a(16, stringArray);
                        } else {
                            stringArray[0] = n.a(n2);
                            string = r.a(35, stringArray);
                        }
                        int n3 = 89;
                        if (n2 == 0) {
                            n3 = -1;
                        }
                        this.b.a(new f(string + "\n" + string2, n3, this.b, -1));
                    }
                    break;
                }
                this.b.a(new f(r.a(4), -1, this.b, -1));
                break;
            }
            case 32: {
                this.b.d(36);
                this.b.a(9, -3);
                this.b.b(13, -2);
                this.b.b(6, r);
                this.b.b(7, s);
                this.b.b(15, 2);
                this.b.b(16, 2);
                this.b.b(12, p.c - t);
                this.b.b(11, p.b - (u << 1) + 4);
                this.b.b(9, u - 2);
                this.b.a(r.a(83), -1, 1);
                this.b.a(0, r.a(47), 0, 18, true);
                this.b.a(new f(r.a(5), -1, this.b, -1));
                break;
            }
            case 33: {
                this.b.d(36);
                this.b.a(9, -3);
                this.b.b(13, -2);
                this.b.b(6, r);
                this.b.b(7, s);
                this.b.b(15, 2);
                this.b.b(16, 2);
                this.b.b(12, p.c - t);
                this.b.b(11, p.b - (u << 1) + 4);
                this.b.b(9, u - 2);
                this.b.a(r.a(84), -1, 1);
                this.b.a(0, r.a(47), 0, 18, true);
                this.b.a(new f(r.a(6), -1, this.b, -1));
                break;
            }
            case 22: {
                this.b.d(36);
                this.b.a(9, -3);
                this.b.b(13, -2);
                this.b.b(15, 2);
                this.b.b(16, 2);
                this.b.b(6, r);
                this.b.b(7, s);
                this.b.b(12, p.c - t);
                this.b.b(11, p.b - (u << 1) + 4);
                this.b.b(9, u - 2);
                this.b.a(3, 0);
                this.b.a(2, 32);
                this.b.b(14, p.a(-3) << 1);
                this.b.a(r.a(18), -1, 1);
                this.b.a(1, r.a(44), 0, 17, true);
                this.b.a(new f(r.a(12), -1, this.b, -1));
                this.b.a(new f(r.a(13), 102, this.b, -1));
                this.b.a(new f(r.a(11), -1, this.b, -1));
                this.b.a(new f(r.a(14), 371, this.b, -1));
                this.b.a(new f(r.a(15), 372, this.b, -1));
                break;
            }
            case 34: {
                String string;
                this.b.a(9, -1);
                this.b.b(13, -1);
                this.b.b(15, 2);
                this.b.b(16, 2);
                this.b.b(11, p.b - v + 4);
                this.b.b(12, p.c - v);
                this.b.b(4, 64);
                this.b.b(3, 32);
                this.b.b(2, 0);
                this.b.a("", -1, 1);
                this.b.a(0, r.a(47), 0, 35, true);
                if (i) {
                    this.b.a(new f(a, -1, this.b, -1));
                    i = false;
                    break;
                }
                if (a != null) {
                    string = r.a(n[0], a);
                    a = null;
                } else {
                    string = r.a(n[0]);
                }
                if (string.endsWith("\n\n")) {
                    string = string.substring(0, string.length() - 2);
                } else if (string.startsWith("\n\n")) {
                    string = string.substring(2, string.length());
                }
                a = string;
                this.b.a(new f(string, -1, this.b, -1));
                break;
            }
            case 24: {
                this.b.a(r.a(82), -1, 1);
                this.b.a(0, r.a(50), 0, 9, false);
                this.b.a(1, r.a(46), 0, 17, true);
                break;
            }
            case 20: {
                this.b.d(37);
                this.b.a(9, -3);
                this.b.b(13, -2);
                this.b.b(6, r);
                this.b.b(7, s);
                this.b.a(r.a(86), -1, 1);
                this.b.a(0, r.a(50), 0, 11, false);
                this.b.a(1, r.a(46), 0, 17, true);
                this.b.a(new f(r.a(2), -1, this.b, -1));
                break;
            }
            case 28: {
                this.b.d(37);
                this.b.a(9, -3);
                this.b.b(13, -2);
                this.b.a(5, 256);
                this.b.b(15, 2);
                this.b.b(16, 2);
                this.b.b(11, p.b - v + 4);
                this.b.b(12, p.c - v);
                this.b.b(4, 64);
                this.b.b(3, 32);
                this.b.b(2, 0);
                this.b.a(r.a(87), -1, 1);
                this.b.a(0, r.a(50), 0, 37, false);
                this.b.a(1, r.a(46), 0, 25, true);
                this.b.a(new f(r.a(3), -1, this.b, -1));
                break;
            }
            case 29: {
                this.b.d(37);
                this.b.a(9, -3);
                this.b.b(13, -2);
                A = 18;
                this.b.a(5, 256);
                this.b.b(15, 2);
                this.b.b(16, 2);
                this.b.b(11, p.b - v + 4);
                this.b.b(12, p.c - v);
                this.b.b(4, 64);
                this.b.b(3, 32);
                this.b.b(2, 0);
                this.b.a(r.a(89), -1, 1);
                this.b.a(0, r.a(50), 0, 5, false);
                this.b.a(1, r.a(46), 0, 25, true);
                this.b.a(new f(r.a(3), -1, this.b, -1));
                break;
            }
            case 30: {
                this.b.d(37);
                this.b.a(9, -3);
                this.b.b(13, -2);
                A = 17;
                this.b.a(5, 256);
                this.b.b(15, 2);
                this.b.b(16, 2);
                this.b.b(11, p.b - v + 4);
                this.b.b(12, p.c - v);
                this.b.b(4, 64);
                this.b.b(3, 32);
                this.b.b(2, 0);
                this.b.a(r.a(82), -1, 1);
                this.b.a(0, r.a(50), 0, 5, false);
                this.b.a(1, r.a(46), 0, 25, true);
                this.b.a(new f(r.a(3), -1, this.b, -1));
                break;
            }
            case 25: {
                this.b.d(37);
                this.b.a(9, -3);
                this.b.b(13, -2);
                this.b.a(5, 256);
                this.b.b(15, 2);
                this.b.b(16, 2);
                this.b.b(11, p.b - v + 4);
                this.b.b(12, p.b - v);
                this.b.b(4, 64);
                this.b.b(3, 32);
                this.b.b(5, 128);
                this.b.a(18, 16742400);
                this.b.b(2, 0);
                this.b.a(r.a(85), -1, 1);
                this.b.a(0, r.a(49), 0, -2, true);
                this.b.a(1, r.a(48), 0, 30, true);
                this.b.a(new f(r.a(38), -1, this.b, 8));
                this.b.a(new f(r.a(37), -1, this.b, 28));
                this.b.a(new f(r.a(39), -1, this.b, 29));
                this.b.a(o);
                break;
            }
            case 18: {
                this.b.a(0, r.a(49), 0, 7, true);
                this.b.a(1, r.a(44), 0, 17, true);
                n.a(this.b);
            }
        }
        this.a = this.b;
        this.b.b();
        p.d(2);
        p.c();
    }

    private void b() {
        this.a = null;
        p.b();
        p.a(1, "", 1);
        p.d(2);
        p.c();
    }

    private static void a(h h2, boolean bl) {
        boolean bl2 = false;
        if (k != 14) {
            if (l < k) {
                bl2 = true;
            }
            l = k++;
            if (m == 0 || bl2) {
                m = 650;
            }
            n = 0;
            if (bl) {
                n.a(h2);
            }
        }
    }

    private static void a(h h2) {
        if (n.b(k)) {
            h2.a(0, r.a(49), 0);
            return;
        }
        h2.a(0, null, 0);
    }

    public static boolean a(h h2, int n2, int n3, int n4, int n5, int n6) {
        Graphics graphics = p.a();
        int n7 = p.a * p.a();
        DirectGraphics directGraphics = DirectUtils.getDirectGraphics((Graphics)graphics);
        if ((h2.a == 34 || h2.a == 25 || h2.a == 28 || h2.a == 29 || h2.a == 30) && n2 == 1) {
            int[] nArray = q.a;
            int[] nArray2 = q.b;
            nArray[0] = n3;
            nArray2[0] = n4;
            nArray[1] = n3 + n5;
            nArray2[1] = n4;
            nArray[2] = n3 + n5;
            nArray2[2] = n4 + n6;
            nArray[3] = n3;
            nArray2[3] = n4 + n6;
            directGraphics.fillPolygon(nArray, 0, nArray2, 0, 4, 0x55000000);
            p.a(n3, n4, 311);
            p.a(n3 + n5, n4, 312);
            p.a(n3, n4 + n6, 309);
            p.a(n3 + n5, n4 + n6, 310);
            n.a(directGraphics);
            return false;
        }
        if (h2.a == 18) {
            if (n2 == 1) {
                n.a(graphics);
                n2 = p.b >> 1;
                int n8 = p.c >> 1;
                int n9 = p.b(331, 2);
                n3 = (short)n9 + n8;
                n9 = p.b(331, 3);
                n4 = (short)n9 + n8;
                n.a(n2, n8, graphics);
                if (k != 0) {
                    p.a(3, n8, 326, 6);
                }
                if (k != 14) {
                    p.a(p.b - 3, n8, 2, 10);
                }
                n5 = 0;
                n9 = 0;
                if (m < n) {
                    if ((m += n7) > n) {
                        m = n;
                    }
                    n5 = k;
                    n9 = l;
                } else if (m > n) {
                    if ((m -= n7) < n) {
                        m = n;
                    }
                    n5 = l;
                    n9 = k;
                }
                if (m > 400 && m < 650) {
                    n9 = (m - 400 << 1) / 250;
                    n.a(n2, n8, n5, n3, n4);
                    p.b(n2, n8, 442, n9);
                } else if (m <= 400 && m > 0) {
                    n6 = n2 - 119 - 239 + 22;
                    n7 = n2 + 120 + -30;
                    int n10 = n2 - 119 + 22;
                    int n11 = n7;
                    n7 -= 25;
                    n7 -= n6;
                    n6 = m * n7 / 400 + n6;
                    n7 = m * (n11 -= n10) / 400 + n10;
                    n10 = n8 - 158 + -3;
                    graphics.setClip(0, 0, n6 + 3, p.c);
                    n.a(n2, n8, n5, n3, n4);
                    graphics.setClip(n7 - 2, 0, p.b - n7 + 2, p.c);
                    n.a(n2, n8, n9, n3, n4);
                    graphics.setClip(0, 0, p.b, p.c);
                    p.a(n6, n10, 377);
                    graphics.setColor(15394508);
                    n9 = n7 - n6 - 12 - 13;
                    graphics.fillRect(n6 + 12, n10, n9, 295);
                    graphics.setColor(0);
                    graphics.fillRect(n6 + 12, n10, n9, 1);
                    graphics.fillRect(n6 + 12, n10 + 307 - 1 + -12, n9, 1);
                    p.a(n6 + 12 + n9, n10, 378);
                    p.a(n2, n8, 376);
                } else {
                    n.a(n2, n8, k, n3, n4);
                }
                p.a(-3, 3);
                p.b(0, 16742400);
                p.b(1, 0);
                p.a(9, 9, 102);
                String string = n.c() + "/" + 450;
                p.a(string, 0, string.length(), 41, 10, 20);
                n.a(directGraphics);
            }
            return false;
        }
        if (h2.a != 25 && h2.a != 28 && h2.a != 29 && h2.a != 30 && h2.a != 15 && h2.a != 34 && n2 == 1) {
            n.a(graphics);
            n2 = p.b >> 1;
            int n12 = p.c >> 1;
            if (h2.a == 17) {
                int n13 = p.b(332, 0);
                n3 = n13 >> 16;
                n4 = (short)n13;
                n13 = p.b(332, 1);
                n5 = n13 >> 16;
                n13 = (short)n13;
                n6 = n2 - 117;
                n7 = n12 - 157;
                int n14 = n2 + n5;
                int n15 = n7;
                graphics.setColor(6570800);
                graphics.fillRect(n2 + n3, n12 + n4, n5 - n3, n13 - n4);
                p.a(n2, n12, 332);
                graphics.setColor(5580559);
                graphics.drawRect(n6 + 2, n7 + 2, (n14 -= n6) - 4, 296);
                graphics.drawRect(n6 + 3, n7 + 3, n14 - 6, 294);
                graphics.setColor(3610889);
                graphics.drawRect(n6, n7, n14, 300);
                graphics.drawRect(n6 + 1, n7 + 1, n14 - 2, 298);
                p.a(n2 + n14 - 117 + 2, n15 - 2, 12, 24, 6);
                p.a(n2 + n14 - 117 + 2, n12 + 300 - 157 + 2, 12, 40);
                p.a(n6 - 5, n12 - 75, 15, 6);
                p.a(n6 - 5, n12 + 75, 15, 6);
            } else {
                n.a(n2, n12, graphics);
            }
            n.a(directGraphics);
            return false;
        }
        if (n2 == 1) {
            n.a(directGraphics);
        }
        if (n2 == 4) {
            return false;
        }
        if (n2 == 2) {
            return false;
        }
        if (n2 == 9) {
            return false;
        }
        if (h2.a != 25 && h2.a != 28 && h2.a != 29 && h2.a != 30 && h2.a != 34 && n2 == 10) {
            graphics.setClip(0, 0, p.b, p.c);
            n2 = b[(int)((System.currentTimeMillis() >> 1) % 360L)] * 5 / 360;
            p.a(n3 - 5 - 4 + n2, n4 + (n6 >> 1), 2, 10);
            p.a(n3 + n5 + 5 + 4 - n2, n4 + (n6 >> 1), 326, 6);
            return false;
        }
        return true;
    }

    private static void a(Graphics graphics) {
        graphics.setColor(4135425);
        graphics.fillRect(0, 0, p.b >> 1, p.c);
        graphics.setColor(6170113);
        graphics.fillRect(p.b >> 1, 0, p.b >> 1, p.c);
        p.a(0, 0, 4);
        p.a(0, 0, 74);
        p.a(0, 0, 75);
        p.a(0, p.c, 3);
    }

    private static int d(int n2) {
        int n3 = -1;
        int n4 = n2 << 2;
        if (f[n4] >= d[n2 * 3]) {
            n3 = 2;
        } else if (f[n4] >= d[n2 * 3 + 1]) {
            n3 = 1;
        } else if (f[n4] >= d[n2 * 3 + 2]) {
            n3 = 0;
        }
        return n3;
    }

    private static int e(int n2) {
        int n3 = -1;
        int n4 = n2 << 2;
        if (f[n4 + 1] <= e[n2 * 3]) {
            n3 = 2;
        } else if (f[n4 + 1] <= e[n2 * 3 + 1]) {
            n3 = 1;
        } else if (f[n4 + 1] <= e[n2 * 3 + 2]) {
            n3 = 0;
        }
        return n3;
    }

    private static void a(int n2, int n3, int n4, int n5, int n6) {
        String string;
        int n7 = n.b(n4);
        if (n7 == 0) {
            p.a(n2, n3, 8);
        } else if (n7 == 2) {
            p.a(n2, n3, 380);
        }
        p.a(n2, n3, (int)g[n4]);
        String[] stringArray = new String[1];
        p.a(-3, 1);
        p.b(0, 0);
        if (!n.b(n4)) {
            p.a(n2, n3, 149);
            if (n.c(n4)) {
                int n8 = 0;
                for (int i2 = 0; i2 < c.length; i2 += 2) {
                    if (n4 != c[i2]) continue;
                    n8 = c[i2 + 1];
                }
                stringArray[0] = "" + n8;
                String string2 = r.a(0, stringArray);
                int n9 = p.a(-3, string2, 0, string2.length());
                int n10 = n9 + 23 + 5;
                n9 = (p.b >> 1) - (n10 >> 1);
                n8 = n9 + n10;
                if (e) {
                    p.a(n8, n6, 102, 24);
                    p.a(string2, 0, string2.length(), n9, n6, 20);
                } else {
                    p.a(n9, n6, 102, 20);
                    p.a(string2, 0, string2.length(), n9 + 23 + 5, n6, 20);
                }
            }
        } else {
            int n11 = p.a(-3) + 1;
            int n12 = p.a(-3) + 23 + 3;
            stringArray[0] = "9999";
            String string3 = r.a(43, stringArray);
            int n13 = p.a(-3, string3, 0, string3.length());
            stringArray[0] = Integer.toString(0);
            int n14 = n4 << 2;
            if (f[n14 + 2] > 0) {
                stringArray[0] = "" + f[n14 + 2];
            }
            n13 = (p.b >> 1) - (n13 >> 1) - 6;
            string3 = "00/00";
            int n15 = p.a(-3, string3, 0, string3.length());
            int n16 = n15 + 23 + 11;
            String string4 = r.a(43, stringArray);
            if (e) {
                p.a(string4, 0, string4.length(), n13 + n16 + 23, n6, 24);
            } else {
                p.a(string4, 0, string4.length(), n13, n6, 20);
            }
            if (!n.c(n4)) {
                stringArray[0] = "" + f[n14] + "/" + 30;
                string4 = stringArray[0];
                if (e) {
                    p.a(n13 + n16, n6 + n11, 102, 20);
                    p.a(string4, 0, string4.length(), n13 - 11 + n16, n6 + n11 + 11, 10);
                } else {
                    p.a(n13, n6 + n11, 102, 20);
                    p.a(string4, 0, string4.length(), n13 + 23 + 11, n6 + n11 + 11, 6);
                }
            }
            if (n.a(13) && !n.c(n4)) {
                int n17;
                int n18 = n.d(n4);
                if (n18 > -1) {
                    n18 = c[n18];
                }
                if ((n17 = n.e(n4)) > -1) {
                    n17 = c[n17];
                }
                stringArray[0] = n.b(f[n14 + 1]);
                string4 = stringArray[0];
                if (e) {
                    p.a(n13 + n16, n6 + n12, 103, 20);
                    p.a(string4, 0, string4.length(), n13 - 11 + n16, n6 + n12 + 11, 10);
                } else {
                    p.a(n13, n6 + n12, 103, 20);
                    p.a(string4, 0, string4.length(), n13 + 23 + 11, n6 + n12 + 11, 6);
                }
                string4 = "00:00";
                int n19 = p.a(-3, string4, 0, string4.length());
                n19 = n19 + 23 + 11;
                if (e) {
                    if (n18 > -1) {
                        p.a(n13, n6 + n11, n18, 24);
                    }
                    if (n17 > -1) {
                        p.a(n13, n6 + n12, n17, 12);
                    }
                } else {
                    if (n18 > -1) {
                        p.a(n13 + n19 + 11, n6 + n11, n18, 20);
                    }
                    if (n17 > -1) {
                        p.a(n13 + n19 + 11, n6 + n12, n17, 20);
                    }
                }
            }
        }
        p.a(-2, 3);
        p.b(0, 16742400);
        p.b(1, 0);
        if (n.c(n4)) {
            stringArray[0] = n.a(n4);
            string = r.a(16, stringArray);
            p.a(string, 0, string.length(), n2, n5 - p.a(p.f()), 33);
        } else {
            stringArray[0] = n.a(n4);
            string = r.a(35, stringArray);
            p.a(string, 0, string.length(), n2, n5 - p.a(p.f()), 33);
        }
        string = r.a(h[n4]);
        p.a(string, 0, string.length(), n2, n5, 33);
    }

    private static String a(int n2) {
        int n3 = 0;
        for (int i2 = 0; i2 < c.length && n2 >= c[i2]; i2 += 2) {
            ++n3;
        }
        if (n.c(n2)) {
            return "" + n3;
        }
        return "" + (n2 + 1 - n3);
    }

    private static void a(int n2, int n3, Graphics graphics) {
        p.a(n2, n3, 331);
        int n4 = p.b(331, 0);
        int n5 = n4 >> 16;
        short s2 = (short)n4;
        n4 = p.b(331, 1);
        int n6 = n4 >> 16;
        n4 = (short)n4;
        graphics.setColor(16512995);
        graphics.fillRect(n2 + n6, n3 + n4, n5 - n6, s2 - n4);
        graphics.setColor(0);
        graphics.fillRect(n2 + n6 - 2, n3 + n4, 4, s2 - n4);
    }

    public static final void a(int n2) {
        if (z < 5) {
            n.n[n.z] = n2;
            ++z;
        }
    }

    private void c() {
        if (!g && z > 0) {
            this.g(34);
            h = true;
            g = true;
            for (int i2 = 0; i2 < 4; ++i2) {
                n.n[i2] = n[i2 + 1];
            }
            --z;
        }
    }

    public final void b(int n2) {
        if (this.a != null) {
            h h2 = this.a;
            int n3 = n2;
            Object object = this;
            if (h2.a == 18) {
                if (f) {
                    if (a[a] == n3) {
                        if ((a = (byte)(a + 1)) == a.length) {
                            for (int i2 = 0; i2 < 15; ++i2) {
                                n.f(i2);
                            }
                            a = 0;
                            n.a(h2);
                            n.d();
                        }
                    } else if (b[a] == n3) {
                        if ((a = (byte)(a + 1)) == b.length) {
                            for (int i3 = 0; i3 < 15; ++i3) {
                                n.f(i3);
                                n.a(i3, (short)30, (short)1, (short)9999);
                            }
                            a = 0;
                            n.a(h2);
                            n.d();
                        }
                    } else {
                        a = 0;
                    }
                }
                switch (n3) {
                    case 3: {
                        object = h2;
                        n3 = 0;
                        if (k == 0) break;
                        if (l > k) {
                            n3 = 1;
                        }
                        l = k--;
                        if (m == 650 || n3 != 0) {
                            m = 0;
                        }
                        n = 650;
                        n.a((h)object);
                        break;
                    }
                    case 4: {
                        n.a(h2, true);
                    }
                }
            }
            this.a.c(n2);
            return;
        }
        if (this.j == 2) {
            this.e();
            return;
        }
        if (this.j != 3 && this.j == 4) {
            int n4 = n2;
            if (f) {
                if (a[a] == n4 && (g.a[1] == 0 || g.a[1] == 1)) {
                    if ((a = (byte)(a + 1)) == a.length) {
                        g.a[0] = 2;
                        a = 0;
                        g = 30;
                    }
                } else {
                    a = 0;
                }
            }
            switch (n4) {
                case 5: {
                    b = true;
                    return;
                }
                case 20: {
                    if (g.a[1] != 0 || g.a[0] == 3) break;
                    a.f();
                }
            }
        }
    }

    public final int a(int n2) {
        if (this.a != null) {
            this.a.b();
            this.a.c();
        } else {
            if (this.j == 2) {
                return 0;
            }
            if (this.j == 4) {
                if (b) {
                    this.g(25);
                    p.d();
                    return 0;
                }
                if (i) {
                    this.g(34);
                    h = true;
                    g = true;
                    return 0;
                }
                b += p.a;
                a = false;
                if (g.a[0] == 0) {
                    n.a.b = (float[])false;
                    if (g.a[1] != 3) {
                        a += p.a;
                    }
                    if (g.a[1] == 0) {
                        n2 = 0;
                        if (p.a(3)) {
                            a.b();
                            n2 = 1;
                        }
                        if (p.a(4)) {
                            a.c();
                            n2 = 1;
                        }
                        if (p.a(15) || p.a(12) || p.a(1) || p.a(7)) {
                            a.a(false);
                            n2 = 1;
                        }
                        if (n2 != 0) {
                            if (c.c != 1) {
                                c.c = 0;
                            }
                            c.b = 3000;
                        }
                    } else if (g.a[1] == 1) {
                        if (p.a(1)) {
                            a.c();
                        }
                        if (p.a(2)) {
                            a.e();
                        }
                        if (p.a(15) || p.a(7)) {
                            a.f();
                        }
                    }
                    n2 = g.a[4];
                    n.i();
                    if (g.a[4] == 0 && n2 != 0) {
                        a.g();
                    }
                    if (n.a.c.f < n.a.j && g.a[0] == 0) {
                        g.a[0] = 1;
                    }
                    if (g == G && n.c(f)) {
                        g.a[0] = 2;
                    }
                    g.a[4] = (int)n.a.c;
                    g.a[5] = (int)n.a.a;
                    g.a[6] = (int)n.a.b;
                    g.a[3] = n.a.d;
                    n2 = p.a;
                    if (H > 0) {
                        H -= n2;
                        if ((I -= n2) <= 0) {
                            I = Math.abs(a.nextInt() % 200 + 300);
                            l = true;
                        }
                        if (H <= 0) {
                            H = 0;
                            I = 0;
                            l = false;
                            m = !m;
                        }
                        for (int i2 = 0; i2 < u.length; ++i2) {
                            if (p.a(u[i2]) == null) continue;
                            if (m && !l || !m && l) {
                                if (b[i2] == null) continue;
                                p.a((int)u[i2], b[i2]);
                                continue;
                            }
                            if (a[i2] == null) continue;
                            p.a((int)u[i2], a[i2]);
                        }
                    }
                } else if (g.a[0] == 1) {
                    p.c(33, 1);
                    g.a[0] = 3;
                    c = 3000;
                    d = n.a.c.f;
                } else if (g.a[0] == 2) {
                    p.c(34, 1);
                    g.a[0] = 4;
                    c = 3000;
                    a.g();
                    if (n.c(f)) {
                        n.a.a = false;
                    }
                    a.a(20, n.a.c.c, n.a.c.f, 740, 0, 1840, 230);
                } else if (g.a[0] == 3) {
                    a.e();
                    if ((c -= p.a) <= 0) {
                        a.a(h, i);
                        c = true;
                        g.a[0] = 0;
                        n.a.a = -16777216;
                        p.c(n.d(), -1);
                    }
                } else if (g.a[0] == 4) {
                    a.a(true);
                    n.i();
                    if ((c -= p.a) <= 0) {
                        int n3;
                        int n4;
                        n n5 = this;
                        int n6 = n.e(f);
                        int n7 = n.d(f);
                        short s2 = f[(f << 2) + 3];
                        boolean bl = n.a(13);
                        boolean bl2 = n.a();
                        int n8 = a / 1000;
                        if (n8 < 1) {
                            n8 = 1;
                        }
                        if ((n4 = (int)((float)(g * 10000) * a[f] / (float)n8)) > Short.MAX_VALUE) {
                            n4 = Short.MAX_VALUE;
                        }
                        p = (short)n4;
                        n.a(f, (short)g, (short)n8, (short)p);
                        if (!n.c(f)) {
                            n8 = f + 1;
                            if (n.c(n8)) {
                                ++n8;
                            }
                            if (n8 <= 14 && !n.b(n8)) {
                                n.e(n8);
                            }
                        }
                        n8 = 0;
                        for (n4 = 0; n4 < c.length; n4 += 2) {
                            if (n.c() < c[n4 + 1] || n.b(n3 = c[n4])) continue;
                            n.e(n3);
                            n8 = (n4 >> 1) + 1;
                        }
                        if (!bl && n.a(13)) {
                            n5.n = true;
                        }
                        if (!bl2 && n.a()) {
                            n5.o = true;
                        }
                        if ((n4 = n.e(f)) > n6) {
                            n5.J = n4;
                        }
                        if ((n3 = n.d(f)) > n7) {
                            n5.K = n3;
                        }
                        if ((n6 = f[(f << 2) + 3]) > s2) {
                            n5.q = true;
                        }
                        n.d();
                        if (n8 > 0) {
                            n5.p = true;
                            a = new String[]{"" + n8};
                            n.a(1);
                        } else {
                            n5.j();
                        }
                    }
                }
                this.c();
                return 0;
            }
        }
        return 0;
    }

    public final int a(int n2, int n3) {
        Object object = p.a();
        if (this.j == 2) {
            if (this.C < 0) {
                Graphics graphics = object;
                object = this;
                object.b(graphics);
                this.e();
            } else {
                Graphics graphics = object;
                object = this;
                graphics.setColor(r[object.C]);
                graphics.fillRect(0, 0, p.b, p.c);
                int n4 = p[object.C];
                p.a((p.b - p.a(n4, 0)) / 2 + p.a(n4, 2), (p.c - p.a(n4, 1)) / 2 + p.a(n4, 3), n4);
                if (System.currentTimeMillis() - this.a > (long)q[this.C]) {
                    this.e();
                }
            }
            return 0;
        }
        if (n3 == 1) {
            if (this.j == 4) {
                object = p.a();
                DirectGraphics directGraphics = DirectUtils.getDirectGraphics((Graphics)object);
                object.setClip(0, 0, x, y);
                Object object2 = object;
                int n5 = n.b(f);
                short[] sArray = i;
                short[] sArray2 = j;
                short[] sArray3 = k;
                short[] sArray4 = l;
                int n6 = 5413606;
                int n7 = 7460351;
                if (n5 == 1) {
                    sArray = m;
                    sArray2 = n;
                    sArray3 = o;
                    sArray4 = p;
                    n6 = 7263689;
                    n7 = 0x9FFFFF;
                } else if (n5 == 2) {
                    sArray = q;
                    sArray2 = r;
                    sArray3 = s;
                    sArray4 = t;
                    n6 = 7737588;
                    n7 = 131610;
                }
                if (n.c(f)) {
                    n6 = 11796669;
                    n7 = 16616467;
                }
                int n8 = n6;
                Object object3 = object2;
                int n9 = p.c / 40;
                int n10 = p.c;
                int n11 = p.b;
                n6 = 0;
                n6 = 0;
                n6 = n8;
                int n12 = n8 >> 16 & 0xFF;
                int n13 = n6 >> 8 & 0xFF;
                n6 &= 0xFF;
                int n14 = n7 >> 16 & 0xFF;
                int n15 = n7 >> 8 & 0xFF;
                n7 &= 0xFF;
                int n16 = 65536 / n10;
                n14 = (n14 - n12) * n16 * n9;
                n15 = (n15 - n13) * n16 * n9;
                n7 = (n7 - n6) * n16 * n9;
                n12 <<= 16;
                n13 <<= 16;
                n6 <<= 16;
                for (n16 = 0; n16 < n10; n16 += n9) {
                    n.a((n12 >> 16 << 16) + (n13 >> 16 << 8) + (n6 >> 16), object3);
                    object3.fillRect(0, n16 + 0, n11, n9);
                    n12 += n14;
                    n13 += n15;
                    n6 += n7;
                }
                a.setSeed(f + 1);
                if (n5 == 2) {
                    n.a(sArray4, 5, 100, 600, 100, -80, 50, 2, -1, object2);
                    n.a(sArray3, 20, 100, p.a((int)sArray3[0], 0), 0, -50, 0, 3, 1380668, object2);
                    n.a(sArray2, 50, 100, p.a((int)sArray2[0], 0), 0, -30, 0, 3, 131610, object2);
                } else {
                    n.a(sArray3, 20, 100, 200, 140, -200, 200, 5, -1, object2);
                    n.a(sArray2, 50, 100, 100, 70, -70, 300, 5, -1, object2);
                    n.a(sArray, 80, 100, 150, 100, -70, 300, 5, -1, object2);
                }
                a.setSeed(System.currentTimeMillis());
                k.a(a, object, directGraphics);
                object.setClip(0, 0, p.b, p.c);
                p.a(-3, 1);
                p.b(0, 0);
                p.a(0, 0, 102);
                int n17 = 32 + n.a(32, 7, g, 4, false);
                p.a(n17, 7, 101);
                n.a(n17 += 9, 7, G, 4, false);
                p.a(p.b, 0, 103);
                n17 = a / 1000;
                int n18 = n17 / 60;
                int n19 = n17 % 60;
                n17 = p.b - 25 - 6 - 1;
                n17 -= n.a(n17, 7, n19, 8, true);
                p.a(n17, 7, 100, 24);
                n.a(n17 -= 5, 7, n18, 8, false);
                l = false;
            }
            if (this.a != null) {
                this.a.d();
            }
        } else if (n3 == 2) {
            this.b((Graphics)object);
        }
        return 0;
    }

    public static void a(String string, int n2, int n3, int n4, int n5) {
        if (n2 == 1) {
            p.a(n3, n4, (n5 | 0x10) == 16 ? 151 : 150, n5);
            return;
        }
        p.a(-3, 3);
        p.b(0, 16742400);
        p.b(1, 0);
        p.a(string, 0, string.length(), n3, n4, n5);
    }

    public static int b() {
        return p.a(-3) + 4;
    }

    public final void c(int n2) {
        if (n2 == 1) {
            p.e(9);
            p.e(14);
            p.e(15);
            p.e(37);
            p.e(36);
            p.e(-1);
            p.e(-2);
            p.e(-3);
            p.a(0);
        }
        if (n2 == 3 && this.j == 4) {
            b = true;
            if (g) {
                i = true;
                g = false;
                this.b();
                return;
            }
            if (!i) {
                a = null;
            }
        }
    }

    /*
     * WARNING - void declaration
     * Enabled force condition propagation
     * Lifted jumps to return sites
     */
    public final int b(int n2, int n3) {
        if (n2 != 17 && n2 != 25) {
            o = 0;
        }
        switch (n2) {
            case 0: {
                p.a(2);
                return 0;
            }
            case 2: {
                if (n3 == 0) {
                    return 1;
                }
                if (n3 == 1) {
                    return 2;
                }
                if (n3 != 2) return 0;
                if (this.a == null || this.a.a == 14) {
                    p.a(true);
                    p.a(1);
                    return 0;
                } else {
                    this.g(19);
                }
                return 0;
            }
            case 1: {
                if (n3 >= 0 && n3 < o.length) {
                    if (n3 == 0) {
                        this.C = -1;
                        this.j = 2;
                    }
                    p.e(o[n3]);
                    return n3 + 1;
                }
                if (n3 != o.length) return 0;
                n.f();
                if (p.c()) {
                    p.g(0);
                }
                byte[] byArray = p.a("game");
                boolean bl = false;
                if (byArray != null) {
                    int n4;
                    for (n4 = 0; n4 < f.length; ++n4) {
                        void var2_6;
                        n.f[n4] = (short)((byArray[++var2_6] & 0xFF) << 8 | byArray[++var2_6] & 0xFF);
                    }
                    if (n.a(13)) return 0;
                    for (n4 = 14; n4 >= 0; --n4) {
                        if (!n.b(n4) || n.c(n4)) continue;
                        k = n4;
                        return 0;
                    }
                    return 0;
                }
                for (int i2 = 0; i2 < f.length; ++i2) {
                    n.f[i2] = 0;
                }
                n.e(0);
                return 0;
            }
            case 6: 
            case 15: {
                if (n3 == 0) {
                    p.d();
                    p.f(10);
                    p.f(13);
                    p.e(v[f]);
                    p.e(v[15]);
                    p.e(2);
                    p.e(1);
                    p.e(6);
                    p.e(5);
                    p.e(24);
                    p.e(0);
                    p.e(3);
                    p.e(26);
                    p.e(20);
                    p.e(19);
                    p.e(16);
                    p.e(25);
                    p.e(27);
                    p.e(21);
                    p.e(23);
                    p.e(22);
                    n2 = n.b(f);
                    if (n2 == 0) {
                        p.e(12);
                        p.e(17);
                        p.e(18);
                    }
                    if (n2 == 1) {
                        p.e(11);
                        p.e(17);
                        p.e(18);
                    }
                    if (n2 == 2) {
                        p.e(8);
                        p.e(4);
                        p.e(17);
                    }
                    a = Image.createImage((int)(c.b[0] * 3), (int)(c.b[0] * 3));
                    a = a.getGraphics();
                    a = new int[a.getWidth() * a.getHeight()];
                    b = Image.createImage((int)112, (int)26);
                    b = b.getGraphics();
                    b = new int[b.getWidth() * b.getHeight()];
                    return 1;
                }
                if (n3 != 1) return 0;
                if (!this.k) {
                    int n5;
                    void var2_8;
                    this.k = true;
                    b = false;
                    p.b(2);
                    p.c(150 / p.a());
                    boolean bl = false;
                    while (var2_8 < 5) {
                        n.n[var2_8] = -1;
                        ++var2_8;
                    }
                    z = 0;
                    g = false;
                    i = false;
                    d = n.a();
                    k.p = 90;
                    k.q = 140;
                    a = 0;
                    b = 0;
                    g = 0;
                    p = 0;
                    g = false;
                    a = false;
                    h = false;
                    k.g.c = p.b << 15;
                    k.g.f = p.c << 15;
                    byte[] byArray = (byte[])p.a(v[15]);
                    int n6 = 8;
                    short s2 = k.a(byArray, 8);
                    n2 = s2;
                    a = new k[s2];
                    n6 += 6;
                    ++n6;
                    n2 = byArray[14];
                    int n7 = 0;
                    int n8 = 0;
                    while (n2 != 127) {
                        n5 = k.a(byArray, n6);
                        n6 += 2;
                        switch (n2) {
                            case 4: {
                                q q2 = new q();
                                q2.a((short)n7);
                                q2.a(byArray, n6);
                                if (q2.a > n8) {
                                    n8 = q2.a;
                                }
                                int n9 = n7;
                                n7 = (short)(n7 + 1);
                                n.a[n9] = q2;
                            }
                        }
                        n6 += n5;
                        n2 = byArray[n6++];
                    }
                    n2 = n8;
                    byte[] byArray2 = (byte[])p.a(v[f]);
                    if (byArray2 != null) {
                        short s3;
                        n6 = 8;
                        short s4 = k.a(byArray2, 8);
                        D = s4;
                        b = new k[s4];
                        short s5 = k.a(byArray2, 12);
                        E = s5;
                        a = new g[s5];
                        n6 += 6;
                        ++n6;
                        n7 = byArray2[14];
                        n8 = 0;
                        n5 = 0;
                        G = 0;
                        while (n7 != 127) {
                            s3 = k.a(byArray2, n6);
                            n6 += 2;
                            switch (n7) {
                                case 4: {
                                    q q3 = new q();
                                    q3.a((short)n8);
                                    q3.a(byArray2, n6);
                                    if (q3.a > n2) {
                                        n2 = q3.a;
                                    }
                                    int n10 = n8;
                                    n8 = (short)(n8 + 1);
                                    n.b[n10] = q3;
                                    break;
                                }
                                case 6: {
                                    g g2 = new g();
                                    g2.a((short)n8);
                                    g2.a(byArray2, n6);
                                    int n11 = n8;
                                    n8 = (short)(n8 + 1);
                                    n.b[n11] = g2;
                                    n.a[n5++] = g2;
                                    break;
                                }
                                case 8: {
                                    a = new c(true);
                                    a.a((short)n8);
                                    a.a(byArray2, n6);
                                    a.a();
                                    int n12 = n8;
                                    n8 = (short)(n8 + 1);
                                    n.b[n12] = a;
                                    h = n.a.c.c;
                                    i = n.a.c.f;
                                    break;
                                }
                                case 9: {
                                    n.b[n8] = new s();
                                    b[n8].a((short)n8);
                                    b[n8].a(byArray2, n6);
                                    int n13 = n8;
                                    n8 = (short)(n8 + 1);
                                    b[n13].a();
                                    break;
                                }
                                case 10: {
                                    i i3 = new i();
                                    i3.a((short)n8);
                                    i3.a(byArray2, n6);
                                    i3.a();
                                    if (i3.a > n2) {
                                        n2 = i3.a;
                                    }
                                    int n14 = n8;
                                    n8 = (short)(n8 + 1);
                                    n.b[n14] = i3;
                                    break;
                                }
                                case 11: {
                                    l l2 = new l();
                                    l2.a((short)n8);
                                    l2.a(byArray2, n6);
                                    l2.a();
                                    int n15 = n8;
                                    n8 = (short)(n8 + 1);
                                    n.b[n15] = l2;
                                    break;
                                }
                                case 12: {
                                    e e2 = new e();
                                    e2.a((short)n8);
                                    e2.a(byArray2, n6);
                                    e2.a();
                                    int n16 = n8;
                                    n8 = (short)(n8 + 1);
                                    n.b[n16] = e2;
                                    break;
                                }
                                case 13: {
                                    a a2 = new a();
                                    a2.a((short)n8);
                                    a2.a(byArray2, n6);
                                    a2.a();
                                    int n17 = n8;
                                    n8 = (short)(n8 + 1);
                                    n.b[n17] = a2;
                                    ++G;
                                    break;
                                }
                                case 14: {
                                    c c2 = new c(false);
                                    c2.a((short)n8);
                                    c2.a(byArray2, n6);
                                    c2.a();
                                    int n18 = n8;
                                    n8 = (short)(n8 + 1);
                                    n.b[n18] = c2;
                                    break;
                                }
                                case 15: {
                                    m m2 = new m();
                                    m2.a((short)n8);
                                    m2.a(byArray2, n6);
                                    m2.a();
                                    int n19 = n8;
                                    n8 = (short)(n8 + 1);
                                    n.b[n19] = m2;
                                    ++G;
                                    break;
                                }
                                default: {
                                    n.b[n8] = new k();
                                    b[n8].a((short)n8);
                                    int n20 = n8;
                                    n8 = (short)(n8 + 1);
                                    b[n20].a(byArray2, n6);
                                }
                            }
                            n6 += s3;
                            n7 = byArray2[n6++];
                        }
                        k.a(b);
                        a = b[0];
                        b = null;
                        a = new a();
                        int n21 = n8;
                        n8 = (short)(n8 + 1);
                        a.a((short)n21);
                        k k2 = a;
                        a a3 = a;
                        a3.b(k2);
                        a3.c.a = 65536;
                        a3.c.b = 0;
                        a3.c.c = Integer.MAX_VALUE;
                        a3.c.d = 0;
                        a3.c.e = 65536;
                        a3.c.f = Integer.MAX_VALUE;
                        a3.b.a(a3.c);
                        a3.a();
                        int n22 = n8;
                        n8 = (short)(n8 + 1);
                        c.a((short)n22);
                        c.c(a);
                        int n23 = n8;
                        n8 = (short)(n8 + 1);
                        b.a((short)n23);
                        b.c(a);
                        int n24 = n8;
                        n8 = (short)(n8 + 1);
                        e.a((short)n24);
                        e.c(a);
                        int n25 = n8;
                        n8 = (short)(n8 + 1);
                        f.a((short)n25);
                        f.c(a);
                        int n26 = n8;
                        n8 = (short)(n8 + 1);
                        g.a((short)n26);
                        g.c(a);
                        int n27 = n8;
                        n8 = (short)(n8 + 1);
                        a.a((short)n27);
                        a.c(a);
                        int n28 = n8;
                        n8 = (short)(n8 + 1);
                        d.a((short)n28);
                        d.c(a);
                        int n29 = n8;
                        n8 = (short)(n8 + 1);
                        h.a((short)n29);
                        h.c(a);
                        i.a((short)n8);
                        i.c(a);
                        int n30 = n2;
                        q.a = new int[n30];
                        q.b = new int[n30];
                        int[] nArray = new int[73];
                        g.a = nArray;
                        nArray[1] = 0;
                        g.a[0] = 0;
                        g.a[2] = 0;
                        g.a[7] = 0;
                        p.f(v[f]);
                        p.f(v[15]);
                        k.a = a;
                        k.m();
                        F = k.f.f;
                        n.g();
                        n.b.a = -1;
                        n.c.a = -1;
                        n.e.a = -1;
                        n.f.a = -1;
                        n.g.a = -1;
                        n.a.a = -1;
                        n.d.a = -1;
                        n.h.a = -1;
                        n.i.a = -1;
                        n.a.a = -16777216;
                        s3 = 1;
                        for (n7 = 0; n7 < c.length; n7 += 2) {
                            int n31 = c[n7];
                            if (!n.a(n31)) continue;
                            s3 = 0;
                        }
                        if (s3 != 0 && n.c(f)) {
                            n.a(11);
                        }
                        c.c = 0;
                        c.b = 3000;
                    }
                }
                this.j = 4;
                this.b();
                if (b) return 0;
                p.c(n.d(), -1);
                return 0;
            }
            case 5: {
                if (n3 == 0) {
                    p.d();
                    n.f();
                    return 1;
                }
                if (n3 != 1) return 0;
                this.j = 3;
                this.g(A);
                p.c(28, -1);
                return 0;
            }
            case 36: {
                if (n3 == 0) {
                    p.f(o[o.length - 1]);
                    return 1;
                }
                if (n3 != 1) return 0;
                this.j = 3;
                p.c(28, -1);
                this.g(17);
                return 0;
            }
            case 21: 
            case 22: {
                o = this.b.a();
                this.g(n2);
                return 0;
            }
            case 11: {
                for (n2 = 0; n2 < f.length; ++n2) {
                    if ((n2 + 1) % 4 == 0) continue;
                    n.f[n2] = 0;
                }
                n.e(0);
                k = 0;
                n.d();
                this.k = false;
                this.g(18);
                return 0;
            }
            case 27: {
                return 0;
            }
        }
        return 0;
    }

    private static int d() {
        int n2 = 31;
        if (n.b(f) == 1) {
            n2 = 30;
        }
        if (n.b(f) == 0) {
            n2 = 29;
        }
        if (f == 4 || f == 9 || f == 14) {
            n2 = 35;
        }
        if (f == 3 || f == 8 || f == 13) {
            n2 = 32;
        }
        return n2;
    }

    public static int b(int n2) {
        int n3 = 2;
        if (n2 <= 9) {
            n3 = 1;
        }
        if (n2 <= 4) {
            n3 = 0;
        }
        return n3;
    }

    public final void d(int n2) {
        if (n2 != 17 && n2 != 25) {
            o = 0;
        }
        switch (n2) {
            case 2: 
            case 5: 
            case 6: 
            case 11: 
            case 15: 
            case 21: 
            case 22: 
            case 27: {
                p.a(n2);
                return;
            }
            case 10: 
            case 20: 
            case 28: 
            case 29: 
            case 30: {
                o = this.b.a();
            }
            case 17: 
            case 18: 
            case 19: 
            case 23: 
            case 24: 
            case 25: 
            case 26: 
            case 31: 
            case 32: 
            case 33: {
                this.g(n2);
                return;
            }
            case 9: {
                p.a();
                return;
            }
            case 35: {
                g = false;
                this.b();
                if (!this.p) break;
                this.p = false;
                this.j();
                return;
            }
            case 13: {
                return;
            }
            case 37: {
                n.h();
            }
            case 7: {
                if (!n.b(k)) break;
                m = n;
                this.k = false;
                f = k;
                p.a(6);
                return;
            }
            case 8: {
                b = false;
                this.b();
                if (b) break;
                p.c(n.d(), -1);
            }
        }
    }

    private static void d() {
        byte[] byArray = new byte[f.length << 1];
        for (int i2 = 0; i2 < f.length; ++i2) {
            byArray[i2 << 1] = (byte)(f[i2] >> 8);
            byArray[(i2 << 1) + 1] = (byte)f[i2];
        }
        p.a("game", byArray);
    }

    private void b(Graphics graphics) {
        if (j) {
            graphics.setColor(7352325);
            graphics.fillRect(0, 0, p.b, p.c);
            graphics.setColor(0);
            graphics.fillRect((p.b - 60) / 2, (p.c - 10) / 2, 60, 10);
            graphics.setColor(4660480);
            graphics.fillRect((p.b - 60) / 2 + 1, (p.c - 10) / 2 + 1, 58, 8);
            graphics.setColor(11460924);
            graphics.fillRect((p.b - 60) / 2 + 1, (p.c - 10) / 2 + 1, 60 * this.B / 20 - 2, 8);
            ++this.B;
            if (this.B > 20) {
                this.B = 0;
                return;
            }
        } else {
            graphics.setColor(0xFFFFFF);
            graphics.fillRect(0, 0, p.b, p.c);
        }
    }

    /*
     * Enabled force condition propagation
     * Lifted jumps to return sites
     */
    private void e() {
        if (this.C + 1 < o.length) {
            if (!p.b(o[this.C + 1])) return;
            j = true;
            ++this.C;
            this.a = System.currentTimeMillis();
            if (this.C - 1 <= -1) return;
            p.f(o[this.C - 1]);
            return;
        }
        p.a(36);
    }

    private static void f() {
        a = null;
        a = null;
        a = null;
        b = null;
        b = null;
        b = null;
        q.a = null;
        q.b = null;
        a = null;
        k.a = null;
        a = null;
        a = null;
        a = null;
        a = null;
        b = null;
        p.f(2);
        p.f(1);
        p.f(6);
        p.f(5);
        p.f(24);
        p.f(0);
        p.f(3);
        p.f(26);
        p.f(20);
        p.f(19);
        p.f(16);
        p.f(25);
        p.f(22);
        p.f(27);
        p.f(21);
        p.f(23);
        int n2 = n.b(f);
        if (n2 == 0) {
            p.f(12);
            p.f(17);
            p.f(18);
        }
        if (n2 == 1) {
            p.f(11);
            p.f(17);
            p.f(18);
        }
        if (n2 == 2) {
            p.f(8);
            p.f(4);
        }
        p.e(10);
        p.e(13);
    }

    private static void g() {
        H = 0;
        I = 0;
        l = false;
        m = false;
        try {
            a = new Image[u.length];
            b = new Image[u.length];
            for (int i2 = 0; i2 < u.length; ++i2) {
                int n2;
                if (p.a(u[i2]) == null) continue;
                n.a[i2] = p.a(u[i2]);
                n.b[i2] = Image.createImage((Image)a[i2]);
                int[] nArray = new int[b[i2].getWidth() * b[i2].getHeight()];
                b[i2].getRGB(nArray, 0, b[i2].getWidth(), 0, 0, b[i2].getWidth(), b[i2].getHeight());
                int[] nArray2 = nArray;
                for (int i3 = 0; i3 < nArray2.length; ++i3) {
                    int n3 = nArray2[i3] >>> 24;
                    n2 = nArray2[i3] >> 16 & 0xFF;
                    int n4 = nArray2[i3] >> 8 & 0xFF;
                    int n5 = nArray2[i3] & 0xFF;
                    int n6 = n2;
                    int n7 = n4;
                    n4 = n5;
                    n2 = n7 + n4 >> 1;
                    n4 = n6 + n4 >> 1;
                    n5 = n6 + n7 >> 1;
                    nArray2[i3] = (n3 << 24) + (n2 << 16) + (n4 << 8) + n5;
                }
                int n8 = b[i2].getWidth();
                n2 = b[i2].getHeight();
                n.b[i2] = Image.createRGBImage((int[])nArray, (int)n8, (int)n2, (boolean)true);
            }
            return;
        }
        catch (Exception exception) {
            return;
        }
    }

    public static int c(int n2) {
        if (m && !l || !m && l) {
            int n3 = n2 >>> 24;
            int n4 = n2 >> 16 & 0xFF;
            int n5 = n2 >> 8 & 0xFF;
            int n6 = n4;
            int n7 = n5;
            n4 = n5 + (n2 &= 0xFF) >> 1;
            n5 = n6 + n2 >> 1;
            n2 = n6 + n7 >> 1;
            n2 = (n3 << 24) + (n4 << 16) + (n5 << 8) + n2;
        }
        return n2;
    }

    private static void a(int n2, Graphics graphics) {
        if (m && !l || !m && l) {
            int n3 = n2 >> 16 & 0xFF;
            int n4 = n2 >> 8 & 0xFF;
            int n5 = n3;
            int n6 = n4;
            n3 = n4 + (n2 &= 0xFF) >> 1;
            n4 = n5 + n2 >> 1;
            n2 = n5 + n6 >> 1;
            n2 = (n3 << 16) + (n4 << 8) + n2;
        }
        graphics.setColor(n2);
    }

    private static void a(short[] sArray, int n2, int n3, int n4, int n5, int n6, int n7, int n8, int n9, Graphics graphics) {
        int n10;
        n.s[0] = 0;
        n3 = 0;
        if (n7 != 0) {
            n3 = a.nextInt() % n7;
        }
        n.t[0] = n6 + n3;
        for (n10 = 1; n10 < n8 + 1; ++n10) {
            n3 = 0;
            if (n5 != 0) {
                n3 = Math.abs(a.nextInt() % n5);
            }
            n.s[n10] = s[n10 - 1] + n3 + n4;
            n3 = 0;
            if (n7 != 0) {
                n3 = a.nextInt() % n7;
            }
            n.t[n10] = n6 + n3;
            n.u[n10] = Math.abs(a.nextInt() % sArray.length);
        }
        n10 = s[n8];
        n3 = n10 - (((k.f.c >> 16) + 33000) * k.g.a >> 16) * n2 / 100 % n10;
        n2 = ((k.f.f - F >> 16) * k.g.a >> 16) * n2 / 100 + p.c;
        for (n4 = 0; n4 < n8; ++n4) {
            if (n4 < 2) {
                p.a(n3 + s[n4], n2 + t[n4], (int)sArray[u[n4]]);
            }
            if (n4 > 2) {
                p.a(n3 + s[n4] - (n10 << 1), n2 + t[n4], (int)sArray[u[n4]]);
            }
            p.a(n3 + s[n4] - n10, n2 + t[n4], (int)sArray[u[n4]]);
        }
        if (n9 != -1) {
            n.a(n9, graphics);
            graphics.fillRect(0, n2 + t[0], p.b, p.c - (n2 + t[0]));
        }
    }

    private static void a(DirectGraphics directGraphics) {
        int n2 = p.c();
        int n3 = p.b;
        int n4 = p.c - n2;
        int[] nArray = v;
        int[] nArray2 = w;
        nArray[0] = 0;
        nArray2[0] = n4;
        nArray[1] = n3 + 0;
        nArray2[1] = n4;
        nArray[2] = n3 + 0;
        nArray2[2] = n4 + n2;
        nArray[3] = 0;
        nArray2[3] = n4 + n2;
        directGraphics.fillPolygon(nArray, 0, nArray2, 0, 4, 0x55000000);
    }

    private static int a(int n2, int n3, int n4, int n5, boolean bl) {
        int n6 = 0;
        n3 = 1;
        int n7 = n4;
        boolean bl2 = bl = n4 < 10 && bl;
        if (n5 == 4) {
            n3 = 0;
        }
        for (n5 = n3; n5 < 2; ++n5) {
            if (n4 == 0) {
                if (n5 == 1) {
                    p.a(n2, 7, (int)w[0], 24);
                }
                n6 += p.a((int)w[0], 0);
            } else {
                while (n4 != 0) {
                    n3 = n4 % 10;
                    n3 = w[n3];
                    if (n5 == 1) {
                        p.a(n2 - n6, 7, n3, 24);
                    }
                    n4 /= 10;
                    n6 += p.a(n3, 0);
                }
            }
            if (n5 != 0) continue;
            n2 += n6;
            n6 = 0;
            n4 = n7;
        }
        if (bl) {
            p.a(n2 - n6, 7, (int)w[0], 24);
            n6 += p.a((int)w[0], 0);
        }
        return n6;
    }

    private static String b(int n2) {
        int n3 = n2 / 60;
        String string = (n2 %= 60) < 10 ? n3 + ":" + "0" + n2 : n3 + ":" + n2;
        return string;
    }

    private static void h() {
        for (int i2 = 0; i2 < u.length; ++i2) {
            if (p.a(u[i2]) == null || a[i2] == null) continue;
            p.a((int)u[i2], a[i2]);
        }
    }

    private static void i() {
        if (h) {
            g.a(a);
            h = false;
            return;
        }
        a.l();
        for (k k2 = a; k2 != null; k2 = k2.a(a)) {
            k2.d();
        }
        g.a(a, a);
        int n2 = g.a[7];
        g.a(a);
        if (g.a[7] != n2) {
            l = false;
            H = 2000;
            I = 0;
        }
        for (k k3 = a; k3 != null; k3 = k3.a(a)) {
            k3.a(a);
        }
        if (c) {
            k.n = 0;
            k.o = 0;
            k.b(true);
            c = false;
            return;
        }
        k.b(false);
    }

    private void j() {
        n.h();
        this.k = false;
        p.a(5);
        A = 31;
        g.a[0] = 0;
    }

    static {
        p.a = e = r.a(90).equals("1");
        a = new byte[]{10, 10, 10};
        b = new byte[]{11, 11, 11};
        String string = p.a.getAppProperty("Cheats");
        f = string != null && string.equals("On");
        c = new short[]{314, 315, 389};
        a = new Random(System.currentTimeMillis());
        k = 0;
        l = 0;
        m = 0;
        n = 0;
        o = 0;
        a = new float[]{0.937f, 0.969f, 0.659f, 0.937f, 1.412f, 0.969f, 2.121f, 1.298f, 1.298f, 1.011f, 1.298f, 1.921f, 1.298f, 0.712f, 1.195f};
        c = new int[]{4, 60, 9, 200, 14, 400};
        d = new int[]{3, 8};
        d = new short[]{30, 29, 26, 30, 28, 26, 30, 25, 21, 30, 25, 20, 30, 30, 30, 30, 28, 25, 30, 28, 22, 30, 27, 20, 30, 27, 24, 30, 30, 30, 30, 29, 27, 30, 28, 19, 30, 25, 20, 30, 25, 20, 30, 30, 30};
        e = new short[]{30, 40, 50, 36, 45, 55, 35, 40, 48, 32, 42, 50, 9999, 9999, 9999, 34, 45, 60, 75, 85, 95, 42, 46, 55, 45, 55, 65, 9999, 9999, 9999, 45, 50, 58, 60, 70, 80, 48, 54, 60, 24, 34, 44, 9999, 9999, 9999};
        f = new short[60];
        g = new short[]{359, 363, 364, 365, 328, 366, 367, 368, 369, 329, 370, 360, 361, 362, 330};
        h = new short[]{20, 24, 25, 26, 32, 27, 28, 29, 30, 33, 31, 21, 22, 23, 34};
        i = new short[]{388};
        j = new short[]{251};
        k = new short[]{252};
        l = new short[0];
        m = new short[]{373};
        n = new short[]{161};
        o = new short[]{159, 160};
        p = new short[0];
        q = new short[]{352};
        r = new short[]{113};
        s = new short[]{114};
        t = new short[]{353};
        u = new short[]{388, 373, 145, 313, 265, 157, 174, 55, 345, 243, 78, 317, 176, 267};
        v = new short[]{39, 40, 41, 42, 51, 43, 44, 45, 46, 52, 47, 48, 49, 50, 53, 54};
        w = 0;
        e = 0;
        x = p.b;
        y = p.c;
        q = 129;
        r = 40;
        s = 5;
        t = 56;
        u = 20;
        v = 40;
        b = new short[360];
        n.a();
        e = new int[]{536};
        f = new int[]{526, 516, 531, 521};
        g = new int[]{390, 420};
        h = new int[]{420, 426};
        i = new int[]{414, 396, 420};
        j = new int[]{420, 426, 402, 408};
        k = new int[]{420, 426};
        l = new int[]{390, 414, 396};
        c = new o(150, 0, 80, 0, 0, 0, 1, e, 4000, 7);
        b = new o(20, 0, -200, 0, 0, 0, 0, f, 800, 7);
        e = new o(10, 0, 0, 0, 0, 30, 2, g, 800, -1);
        f = new o(50, 0, 0, 0, 0, 85, 3, h, 540, -2);
        g = new o(50, 0, 0, 0, 0, 35, 4, i, 1840, -3);
        a = new o(20, 0, 0, 0, 0, 35, 4, j, 1840, -4);
        d = new o(10, 0, 0, 0, 0, 0, 6, k, 1000, -5);
        h = new o(24, 0, 0, 0, 0, 35, 4, l, 2040, -6);
        m = new int[]{437, 432};
        i = new o(150, 0, 0, 0, 0, 0, 7, m, 2000, 15);
        n = new int[5];
        a = null;
        i = false;
        A = 17;
        j = false;
        o = new int[]{7};
        p = new int[]{1};
        q = new int[]{3000};
        r = new int[]{0xFFFFFF};
        c = false;
        s = new int[10];
        t = new int[10];
        u = new int[10];
        v = new int[4];
        w = new int[4];
        w = new short[]{90, 91, 92, 93, 94, 95, 96, 97, 98, 99};
    }
}

