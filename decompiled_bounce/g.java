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
public final class g
extends k {
    public static int[] a;
    private byte[][] a;
    private byte a;
    private byte d;
    private byte e;
    private short a;
    private byte f = 0;
    private byte g = (byte)-1;
    private int a;
    private int b;
    private k[] a = new k[2];
    private k[] b = new k[2];
    private static k[] c;
    private static g[] a;
    private static int c;

    public g() {
        this.c = (k[])3;
    }

    public final int a(byte[] byArray, int n2) {
        n2 = super.a(byArray, n2);
        this.e = (byte)((byArray[n2++] << 8 | byArray[n2++] & 0xFF) << 16);
        this.h = (byArray[n2++] << 8 | byArray[n2++] & 0xFF) << 16;
        this.g = (byte)((byArray[n2++] << 8 | byArray[n2++] & 0xFF) << 16);
        this.f = (byte)((byArray[n2++] << 8 | byArray[n2++] & 0xFF) << 16);
        this.a = byArray[n2++];
        if (this.a == 2) {
            this.g = 0;
        }
        this.d = byArray[n2++];
        this.e = byArray[n2++];
        this.a = g.a(byArray, n2);
        n2 += 2;
        this.f = byArray[n2++];
        this.a = new byte[this.f][];
        for (int i2 = 0; i2 < this.f; ++i2) {
            int n3 = byArray[n2++];
            this.a[i2] = new byte[n3];
            for (int i3 = 0; i3 < n3; ++i3) {
                this.a[i2][i3] = byArray[n2++];
            }
        }
        this.m |= 0x80;
        return n2;
    }

    protected final void a(int n2) {
        switch (this.a) {
            case 3: {
                if (n2 != 2) break;
                this.a = (byte)2;
                return;
            }
            case 0: {
                if (n2 == 2) {
                    this.a = (byte)2;
                    this.g = 0;
                }
                if (n2 != 1) break;
                this.a = 1;
                return;
            }
            case 1: {
                if (n2 != 0) break;
                this.a = 0;
                this.b();
                return;
            }
            case 2: {
                if (n2 == 1) {
                    this.a = 1;
                }
                if (n2 != 3) break;
                this.a = (byte)3;
            }
        }
    }

    private final void b() {
        this.g = (byte)-1;
        for (int i2 = 0; i2 < this.a.length; ++i2) {
            byte[] byArray = this.a[i2];
            byte by = byArray[0];
            switch (by) {
                case 6: {
                    byArray[3] = byArray[1];
                    byArray[4] = byArray[2];
                    break;
                }
                case 16: {
                    byArray[15] = byArray[11];
                    byArray[16] = byArray[12];
                    byArray[17] = byArray[13];
                    byArray[18] = byArray[14];
                    break;
                }
                case 17: {
                    byArray[11] = byArray[7];
                    byArray[12] = byArray[8];
                    byArray[13] = byArray[9];
                    byArray[14] = byArray[10];
                }
            }
        }
    }

    public final void a(Graphics graphics, DirectGraphics directGraphics, d d2) {
    }

    /*
     * WARNING - void declaration
     */
    public static void a(g[] gArray, c c2) {
        void var3_6;
        reference var2_2 = c2.g - c2.e;
        reference var3_4 = c2.h - c2.f;
        reference var2_3 = var2_2 < var3_4 ? var3_4 >> 1 : var2_2 >> 1;
        boolean bl = false;
        while (var3_6 < gArray.length) {
            g g2 = gArray[var3_6];
            if (g2.a != 1) {
                g2.a(k.a);
                k.a.c(d.a);
                d.a.a(c2.b.c, c2.b.f);
                int n2 = d.g;
                int n3 = d.h;
                d.a.a(c2.c.c, c2.c.f);
                int n4 = d.g;
                int n5 = d.h;
                int n6 = g2.e - var2_3;
                int n7 = g2.f - var2_3;
                int n8 = g2.g + var2_3;
                int n9 = g2.h + var2_3;
                int n10 = k.a(n2, n3, n4, n5, n6, n7, n8, n9);
                n2 = n10;
                n2 = n10 | k.a(n4, n5, n6, n7, n8, n9);
                if (n2 != 0) {
                    c c3 = c2;
                    if (g2.a < 2) {
                        g2.a[g2.a] = c3;
                        ++g2.a;
                    }
                }
            }
            ++var3_6;
        }
    }

    private static final boolean a(Object[] objectArray, int n2, Object object) {
        for (int i2 = 0; i2 < n2; ++i2) {
            if (objectArray[i2] != object) continue;
            return true;
        }
        return false;
    }

    /*
     * Unable to fully structure code
     */
    public static void a(g[] var0) {
        g.a = var0;
        for (var1_1 = 0; var1_1 < var0.length; ++var1_1) {
            var2_2 = var0[var1_1];
            if (var2_2.d == 0) {
                for (var3_3 = 0; var3_3 < var2_2.a; ++var3_3) {
                    var4_10 = var2_2.a[var3_3];
                    if (g.a(var2_2.b, var2_2.b, var4_10)) continue;
                    g.c[var3_3] = var4_10;
                }
            } else {
                for (var3_3 = 0; var3_3 < var2_2.b; ++var3_3) {
                    var4_10 = var2_2.b[var3_3];
                    if (g.a(var2_2.a, var2_2.a, var4_10)) continue;
                    g.c[var3_3] = var4_10;
                }
            }
            for (var3_3 = 0; var3_3 < g.c.length; ++var3_3) {
                if (g.c[var3_3] == null || var2_2.a > -1 && g.c[var3_3].a() != var2_2.a) continue;
                var2_2.a(2);
            }
            for (var3_3 = 0; var3_3 < var2_2.a.length; ++var3_3) {
                var2_2.b[var3_3] = var2_2.a[var3_3];
                var2_2.a[var3_3] = null;
                g.c[var3_3] = null;
            }
            var2_2.b = var2_2.a;
            var2_2.a = 0;
        }
        for (var1_1 = 0; var1_1 < var0.length; ++var1_1) {
            do {
                block101: {
                    block102: {
                        if (!(var2_2 = var0[var1_1]).a(n.a) || var2_2.a != 2) break block101;
                        var4_12 = var2_2.a[var2_2.g];
                        var3_5 = var2_2;
                        var5_16 = var4_12[0];
                        switch (var5_16) {
                            case 1: {
                                var4_13 = k.a(var4_12, 1);
                                var3_5 = var3_5.a().a((short)var4_13);
                                if (var3_5 != null) {
                                    var3_5 = (s)var3_5;
                                    for (var4_13 = 0; var4_13 < var3_5.a.length; ++var4_13) {
                                        var5_16 = var3_5.a[var4_13];
                                        var6_20 = -1;
                                        var7_32 = 0;
                                        switch (var5_16) {
                                            case 118: {
                                                var6_20 = 485;
                                                var7_32 = 9999;
                                                break;
                                            }
                                            case 334: {
                                                var6_20 = 474;
                                                var7_32 = 750;
                                                break;
                                            }
                                            case 342: {
                                                var6_20 = 480;
                                                var7_32 = 9999;
                                            }
                                        }
                                        if (var6_20 <= -1) continue;
                                        var3_5.b[var4_13] = var6_20;
                                        var3_5.a[var4_13] = var7_32;
                                    }
                                }
                                v0 = true;
                                break block102;
                            }
                            case 0: {
                                var3_6 = k.a(var4_12, 1);
                                var3_6 = n.a[var3_6];
                                if (!n.a(13)) {
                                    n.a(var3_6);
                                }
                                v0 = true;
                                break block102;
                            }
                            case 25: {
                                n.h = var3_5.b.c;
                                n.i = var3_5.b.f;
                                v0 = true;
                                break block102;
                            }
                            case 26: {
                                var5_16 = k.a(var4_12, 1);
                                var6_21 = var3_5.a().a((short)var5_16);
                                if (var6_21.c == 4) {
                                    var7_33 = (c)var6_21;
                                    var7_33.f += (float)k.a(var4_12, 3);
                                    var7_33.g += (float)k.a(var4_12, 5);
                                }
                                v0 = true;
                                break block102;
                            }
                            case 27: {
                                var5_16 = k.a(var4_12, 1);
                                var6_22 = var3_5.a().a((short)var5_16);
                                if (var6_22.c == 4) {
                                    var7_34 = (c)var6_22;
                                    var7_34.d += (float)k.a(var4_12, 3);
                                    var7_34.e += (float)k.a(var4_12, 5);
                                }
                                v0 = true;
                                break block102;
                            }
                            case 28: {
                                var5_16 = k.a(var4_12, 1);
                                var6_23 = var3_5.a().a((short)var5_16);
                                if (var6_23.c == 4) {
                                    var7_35 = (c)var6_23;
                                    var7_35.a += (float)k.a(var4_12, 3);
                                    var7_35.b += (float)k.a(var4_12, 5);
                                }
                                v0 = true;
                                break block102;
                            }
                            case 3: {
                                g.a[var4_12[1]].a(0);
                                v0 = true;
                                break block102;
                            }
                            case 2: {
                                g.a[var4_12[1]].a(1);
                                v0 = true;
                                break block102;
                            }
                            case 4: {
                                g.a[var4_12[1]].a(2);
                                v0 = true;
                                break block102;
                            }
                            case 5: {
                                g.a[var4_12[1]].a(3);
                                v0 = true;
                                break block102;
                            }
                            case 24: {
                                var5_16 = k.a(var4_12, 1);
                                var6_24 = var3_5.a().a((short)var5_16);
                                if (g.a(var3_5.b, var3_5.b, var6_24)) break;
                                v0 = true;
                                break block102;
                            }
                            case 6: {
                                var5_16 = k.a(var4_12, 3);
                                if (var5_16 >= 0) ** GOTO lbl130
                                var4_12[3] = var4_12[1];
                                var4_12[4] = var4_12[2];
                                v0 = true;
                                break block102;
lbl130:
                                // 1 sources

                                var5_16 = (short)(var5_16 - p.a);
                                var4_14 = 3;
                                var3_5 = var4_12;
                                var4_12[3] = (byte)(var5_16 >> 8);
                                var3_5[4] = (byte)var5_16;
                                break;
                            }
                            case 30: {
                                var5_16 = k.a(var4_12, 1);
                                k.a = var6_25 = var3_5.a().a((short)var5_16);
                                v0 = true;
                                break block102;
                            }
                            case 31: {
                                n.c = var4_12[1] == 1;
                                k.p = k.a(var4_12, 2);
                                k.q = k.a(var4_12, 4);
                                v0 = true;
                                break block102;
                            }
                            case 32: {
                                n.c = false;
                                k.p = 90;
                                k.q = 140;
                                v0 = true;
                                break block102;
                            }
                            case 19: {
                                var5_16 = k.a(var4_12, 1);
                                var6_26 = k.a(var4_12, 3);
                                var7_36 = var3_5.a();
                                var3_5 = k.b.a((short)var5_16);
                                if (var3_5 == null) {
                                    v0 = true;
                                } else {
                                    var5_17 = var7_36.a(var6_26);
                                    if (var5_17 == null) {
                                        v0 = true;
                                    } else {
                                        var3_5.b(var5_17);
                                        var3_5.i();
                                        v0 = true;
                                    }
                                }
                                break block102;
                            }
                            case 20: {
                                var5_16 = k.a(var4_12, 1);
                                var6_27 = var3_5.a();
                                var7_37 = var6_27.a((short)var5_16);
                                if (var7_37 == null) {
                                    v0 = true;
                                } else {
                                    var7_37.i();
                                    var7_37.b(k.b);
                                    v0 = true;
                                }
                                break block102;
                            }
                            case 18: {
                                var5_16 = k.a(var4_12, 1);
                                var6_28 = var3_5.a().a((short)var5_16);
                                if (var6_28 == null) {
                                    v0 = true;
                                } else {
                                    var7_38 = k.a(var4_12, 3);
                                    if (var7_38 < 0) {
                                        var6_28.c.c = k.b(var4_12, 5);
                                        var6_28.c.f = k.b(var4_12, 9);
                                    } else if ((var3_5 = var3_5.a().a(var7_38)) != null) {
                                        var3_5.a(k.a);
                                        var5_16 = k.a.c;
                                        var8_42 = k.a.f;
                                        var6_28.c.c = 0;
                                        var6_28.c.f = 0;
                                        var6_28.f = true;
                                        var6_28.a(k.a);
                                        k.a.c(d.a);
                                        d.a.a(var5_16, var8_42);
                                        var6_28.c.c = d.g;
                                        var6_28.c.f = d.h;
                                    }
                                    var6_28.h();
                                    var6_28.i();
                                    var6_28.a(var6_28.b);
                                    v0 = true;
                                }
                                break block102;
                            }
                            case 16: {
                                var5_16 = k.a(var4_12, 1);
                                var6_29 = var3_5.a().a((short)var5_16);
                                if (var6_29 != null) ** GOTO lbl214
                                v0 = true;
                                break block102;
lbl214:
                                // 1 sources

                                var7_39 = k.b(var4_12, 15);
                                if (var7_39 - (var3_7 = p.a) <= 0) {
                                    var4_12[15] = var4_12[11];
                                    var4_12[16] = var4_12[12];
                                    var4_12[17] = var4_12[13];
                                    var4_12[18] = var4_12[14];
                                    var3_7 = var7_39;
                                }
                                var6_29.c.c += k.b(var4_12, 3) * var3_7;
                                var6_29.c.f += k.b(var4_12, 7) * var3_7;
                                var6_29.h();
                                var6_29.i();
                                if ((var7_39 -= var3_7) > 0) ** GOTO lbl228
                                v0 = true;
                                break block102;
lbl228:
                                // 1 sources

                                g.a(var4_12, 15, var7_39);
                                break;
                            }
                            case 17: {
                                var5_16 = k.a(var4_12, 1);
                                var6_30 = var3_5.a().a((short)var5_16);
                                if (var6_30 != null) ** GOTO lbl236
                                v0 = true;
                                break block102;
lbl236:
                                // 1 sources

                                var7_40 = k.b(var4_12, 11);
                                var3_8 = k.b(var4_12, 7);
                                var5_16 = p.a;
                                if (var7_40 == var3_8) {
                                    g.a(var4_12, 15, var6_30.c.a);
                                    g.a(var4_12, 19, var6_30.c.b);
                                    g.a(var4_12, 23, var6_30.c.d);
                                    g.a(var4_12, 27, var6_30.c.e);
                                }
                                if (var7_40 - var5_16 <= 0) {
                                    var4_12[11] = var4_12[7];
                                    var4_12[12] = var4_12[8];
                                    var4_12[13] = var4_12[9];
                                    var4_12[14] = var4_12[10];
                                    var7_40 = 0;
                                }
                                var8_43 = (float)((long)k.b(var4_12, 3) * (long)(var3_8 - var7_40) / (long)var3_8) / 65536.0f;
                                d.a.a(var8_43);
                                d.a.c = 0;
                                d.a.f = 0;
                                var6_30.c.a = k.b(var4_12, 15);
                                var6_30.c.b = k.b(var4_12, 19);
                                var6_30.c.d = k.b(var4_12, 23);
                                var6_30.c.e = k.b(var4_12, 27);
                                var6_30.c.b(d.a);
                                var6_30.h();
                                var6_30.i();
                                if ((var7_40 -= var5_16) > 0) ** GOTO lbl264
                                v0 = true;
                                break block102;
lbl264:
                                // 1 sources

                                g.a(var4_12, 11, var7_40);
                                break;
                            }
                            case 29: {
                                var5_16 = k.a(var4_12, 1);
                                var6_31 = var3_5.a().a((short)var5_16);
                                if (var6_31 == null) {
                                    v0 = true;
                                } else {
                                    var7_41 = k.b(var4_12, 3);
                                    var3_9 = k.b(var4_12, 7);
                                    if ((var7_41 & 1) > 0) {
                                        var6_31.m &= -32;
                                        var6_31.m |= var3_9 & 31;
                                        var6_31.b = (byte)((var6_31.m & 31) - 16);
                                    }
                                    if ((var7_41 & 32) > 0) {
                                        var6_31.m &= -33;
                                        var6_31.m |= var3_9 & 32;
                                    }
                                    if ((var7_41 & 128) > 0) {
                                        var6_31.m &= -129;
                                        var6_31.m |= var3_9 & 128;
                                        if (var6_31.a() == 5) {
                                            var5_18 = (s)var6_31;
                                            if (var5_18.a[0] == 358) {
                                                var5_18.a(k.a);
                                                var8_44 = k.a.c + (var5_18.e + 0x3C0000);
                                                var4_15 = k.a.f + (var5_18.f + 0x3C0000);
                                                var9_45 = k.a.c + (var5_18.g - 0x3C0000);
                                                var5_19 = k.a.f + (var5_18.h - 0x3C0000);
                                                n.h.a(24, var8_44, var4_15, var9_45, var5_19, 840, 0, 0, 360, 2040, 510);
                                            }
                                        }
                                    }
                                    if ((var7_41 & 256) > 0) {
                                        var6_31.m &= -257;
                                        var6_31.m |= var3_9 & 256;
                                    }
                                    v0 = true;
                                }
                                break block102;
                            }
                            case 23: {
                                if (var3_5.e == 1) {
                                    var3_5.a = 0;
                                    break;
                                }
                                var3_5.a = 1;
                                break;
                            }
                            case 21: {
                                var3_5.g = (byte)(var4_12[1] - 2);
                                v0 = true;
                                break block102;
                            }
                            case 22: {
                                v0 = true;
                                break block102;
                            }
                            case 7: {
                                if (var4_12[1] == 1) {
                                    g.a(var4_12[2], g.a(var4_12, 3));
                                }
                                if (var4_12[1] == 2) {
                                    g.a[var4_12[2]] = g.c(var4_12, 3);
                                }
                                v0 = true;
                                break block102;
                            }
                            case 8: {
                                if (var4_12[1] == 1) {
                                    g.a(var4_12[2], g.a(var4_12[2]) + g.a(var4_12, 3));
                                }
                                if (var4_12[1] == 2) {
                                    v1 = var4_12[2];
                                    g.a[v1] = g.a[v1] + g.c(var4_12, 3);
                                }
                                v0 = true;
                                break block102;
                            }
                            case 9: {
                                if (var4_12[1] == 1) {
                                    g.a(var4_12[2], g.a(var4_12[2]) - g.a(var4_12, 3));
                                }
                                if (var4_12[1] == 2) {
                                    v2 = var4_12[2];
                                    g.a[v2] = g.a[v2] - g.c(var4_12, 3);
                                }
                                v0 = true;
                                break block102;
                            }
                            case 10: {
                                if (var4_12[1] == 1) {
                                    g.a(var4_12[2], g.a(var4_12[2]) * g.a(var4_12, 3));
                                }
                                if (var4_12[1] == 2) {
                                    v3 = var4_12[2];
                                    g.a[v3] = g.a[v3] * g.c(var4_12, 3);
                                }
                                v0 = true;
                                break block102;
                            }
                            case 11: {
                                if (var4_12[1] == 1) {
                                    g.a(var4_12[2], g.a(var4_12[2]) / g.a(var4_12, 3));
                                }
                                if (var4_12[1] == 2) {
                                    v4 = var4_12[2];
                                    g.a[v4] = g.a[v4] / g.c(var4_12, 3);
                                }
                                v0 = true;
                                break block102;
                            }
                            case 12: {
                                if (var4_12[1] != 2) ** GOTO lbl355
                                if (g.a[var4_12[2]] != g.c(var4_12, 3)) ** GOTO lbl-1000
                                v0 = true;
                                break block102;
lbl355:
                                // 1 sources

                                if (var4_12[1] == 1 && g.a(var4_12[2]) == g.a(var4_12, 3)) {
                                    v0 = true;
                                } else lbl-1000:
                                // 2 sources

                                {
                                    var3_5.g = (byte)(var4_12[g.c] - 2);
                                    v0 = true;
                                }
                                break block102;
                            }
                            case 13: {
                                if (var4_12[1] != 2) ** GOTO lbl366
                                if (g.a[var4_12[2]] == g.c(var4_12, 3)) ** GOTO lbl-1000
                                v0 = true;
                                break block102;
lbl366:
                                // 1 sources

                                if (var4_12[1] == 1 && g.a(var4_12[2]) != g.a(var4_12, 3)) {
                                    v0 = true;
                                } else lbl-1000:
                                // 2 sources

                                {
                                    var3_5.g = (byte)(var4_12[g.c] - 2);
                                    v0 = true;
                                }
                                break block102;
                            }
                            case 14: {
                                if (var4_12[1] != 2) ** GOTO lbl377
                                if (g.a[var4_12[2]] >= g.c(var4_12, 3)) ** GOTO lbl-1000
                                v0 = true;
                                break block102;
lbl377:
                                // 1 sources

                                if (var4_12[1] == 1 && g.a(var4_12[2]) < g.a(var4_12, 3)) {
                                    v0 = true;
                                } else lbl-1000:
                                // 2 sources

                                {
                                    var3_5.g = (byte)(var4_12[g.c] - 2);
                                    v0 = true;
                                }
                                break block102;
                            }
                            case 15: {
                                if (var4_12[1] != 2) ** GOTO lbl388
                                if (g.a[var4_12[2]] <= g.c(var4_12, 3)) ** GOTO lbl-1000
                                v0 = true;
                                break block102;
lbl388:
                                // 1 sources

                                if (var4_12[1] == 1 && g.a(var4_12[2]) > g.a(var4_12, 3)) {
                                    v0 = true;
                                } else lbl-1000:
                                // 2 sources

                                {
                                    var3_5.g = (byte)(var4_12[g.c] - 2);
                                    v0 = true;
                                }
                                break block102;
                            }
                            default: {
                                v0 = true;
                                break block102;
                            }
                        }
                        v0 = false;
                    }
                    if (v0) {
                        if (var2_2.g < -1) {
                            var2_2.g = (byte)-1;
                        }
                        var2_2.g = (byte)(var2_2.g + 1);
                        if (var2_2.g >= var2_2.f) {
                            if (var2_2.e == 1) {
                                var2_2.a = 0;
                                var2_2.b();
                            } else {
                                var2_2.a = 1;
                            }
                            v5 = false;
                            continue;
                        }
                        v5 = true;
                        continue;
                    }
                }
                v5 = false;
            } while (v5);
        }
    }

    private static final int c(byte[] byArray, int n2) {
        n2 = byArray[3];
        int n3 = 0;
        c = 4;
        switch (n2) {
            case 32: {
                c += 4;
                n3 = k.b(byArray, 4);
                break;
            }
            case 2: {
                ++c;
                byte by = byArray[4];
                n3 = a[by];
            }
        }
        return n3;
    }

    private static float a(byte by) {
        return Float.intBitsToFloat(a[by]);
    }

    private static final void a(byte by, float f2) {
        g.a[by] = Float.floatToIntBits(f2);
    }

    private static final float a(byte[] byArray, int n2) {
        n2 = byArray[3];
        float f2 = 0.0f;
        c = 4;
        switch (n2) {
            case 16: {
                c += 4;
                n2 = 4;
                long l2 = 0L | (long)byArray[n2];
                l2 <<= 8;
                l2 |= (long)(byArray[n2 + 1] & 0xFF);
                l2 <<= 8;
                l2 |= (long)(byArray[n2 + 2] & 0xFF);
                l2 <<= 8;
                f2 = Float.intBitsToFloat((int)(l2 |= (long)(byArray[n2 + 3] & 0xFF)));
                break;
            }
            case 1: {
                ++c;
                f2 = g.a(byArray[4]);
                break;
            }
            case 32: {
                c += 4;
                f2 = k.b(byArray, 4);
                break;
            }
            case 2: {
                ++c;
                byte by = byArray[4];
                f2 = a[by];
            }
        }
        return f2;
    }

    private static void a(byte[] byArray, int n2, int n3) {
        byArray[n2] = (byte)(n3 >>> 24);
        byArray[n2 + 1] = (byte)(n3 >> 16);
        byArray[n2 + 2] = (byte)(n3 >> 8);
        byArray[n2 + 3] = (byte)n3;
    }

    static {
        c = new k[2];
        c = 0;
    }
}

