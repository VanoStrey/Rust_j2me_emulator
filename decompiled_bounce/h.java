/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  javax.microedition.lcdui.Graphics
 */
import java.io.ByteArrayInputStream;
import java.io.DataInputStream;
import java.util.Vector;
import javax.microedition.lcdui.Graphics;

/*
 * Duplicate member names - consider using --renamedupmembers true
 */
public final class h {
    private Vector a;
    private int b;
    private int c;
    public int a;
    public int[] a;
    public int[] b = null;
    private b a;
    private int d;
    private int e = 0;
    private int f = 0;
    private int g = 1;
    private int[] c = 0;
    private String[] a = null;
    private int[] d = -1;
    private static final int[] e = new int[4];
    private static final int[] f = new int[]{3, 12, 48, 64, 128, 256, 1536, 2048, 4096};
    private static int[] g = new int[]{-1, 7304, -1, -1, 1, 1, 1, 1, 0xFFFFFF, 0, 0, 0x666666, 0, 0, 0x666666, 0xAA7700};
    private static final int[] h = new int[]{3, 12, 16, 32, 64, 128};
    private static int[] i = new int[]{-1, 8, 2, 2, 2, 0, 0, -1, -1, -1, 20, 12, 12, 2, 6, 1, 0x440000, 6170113, 68, 0xA0A0A0, 0, 0x990000, 0};

    public h() {
        this.c = new int[3];
        this.a = new String[3];
        this.d = new int[3];
    }

    public final void a(f f2) {
        int n2 = this.a.size();
        this.a.insertElementAt(f2, n2);
        f2.a = this;
        if (this.b == -1 && this.a(n2)) {
            this.a(n2);
        }
    }

    public final void a() {
        this.a.removeAllElements();
        this.c = 0;
        this.b = -1;
    }

    private boolean a(int n2) {
        f f2 = (f)this.a.elementAt(n2);
        return (n2 = this.a(n2)) >= this.c && n2 + f2.b() <= this.c + this.e();
    }

    public final int a() {
        return this.b;
    }

    public final void a(int n2) {
        if (n2 >= this.a.size()) {
            n2 = this.a.size() - 1;
        }
        if (n2 == this.b) {
            return;
        }
        if (n2 != -1 && !((f)this.a.elementAt(n2)).a()) {
            return;
        }
        int n3 = this.b;
        this.b = n2;
        if (n3 != -1) {
            this.a.elementAt(n3);
        }
        if (n2 != -1) {
            this.a.elementAt(n2);
        }
        if (this.b != -1) {
            n2 = this.a(this.b);
            n3 = n2 + ((f)this.a.elementAt(this.b)).b();
            int n4 = this.e();
            if (n2 < this.c) {
                this.c = n2;
                return;
            }
            if (n3 > this.c + n4) {
                this.c = n3 - n4;
            }
        }
    }

    private f a() {
        f f2 = ((h)((Object)f2)).b != -1 ? (f)((h)((Object)f2)).a.elementAt(((h)((Object)f2)).b) : null;
        if (f2 != null && f2.a) {
            return f2;
        }
        return null;
    }

    public final void a(String string, int n2, int n3) {
        n2 = this.c(0);
        this.a = string != null && string.length() > 0 ? new b(string, Integer.MAX_VALUE, this.c(13), n2 |= 0x100, -1) : null;
        this.d = -1;
        this.f = 0;
    }

    public final void a(int n2, String string, int n3, int n4, boolean bl) {
        this.c[n2] = 0x110000 | (short)n4 & 0xFFFF;
        this.a[n2] = string;
        this.d[n2] = 0;
    }

    public final void a(int n2, String string, int n3) {
        if ((this.c[0] & 0x10000) == 65536) {
            this.a[0] = string;
            this.d[0] = 0;
        }
    }

    public final void b(int n2) {
        this.c[n2] = 0;
        this.a[n2] = null;
        this.d[n2] = 0;
    }

    public final void b() {
        for (int i2 = 0; i2 < 3; ++i2) {
            boolean bl = (short)this.c[i2] == -2;
            f f2 = this.a();
            if ((this.c[i2] & 0x10000) == 0 || (this.c[i2] & 0x100000) == 0 || bl && f2 == null) {
                p.a(i2 + 0, null, 0);
                continue;
            }
            p.a(i2 + 0, this.a[i2], this.d[i2]);
        }
    }

    private int a(int n2) {
        int n3 = 0;
        for (int i2 = 0; i2 < n2; ++i2) {
            n3 += ((f)this.a.elementAt(i2)).b() + this.c(19);
        }
        return n3;
    }

    private static int a(f f2, int n2) {
        int n3 = f2.a(6);
        if (n3 == 512) {
            return n2 - f2.a();
        }
        if (n3 == 1024) {
            return n2 - f2.a() >> 1;
        }
        return 0;
    }

    private int c() {
        int n2 = this.c(11);
        if (n2 > 0) {
            return n2;
        }
        return p.e() - (p.a() ? p.d() : 0);
    }

    public final int b() {
        return this.c() - this.c(15) - this.c(16);
    }

    private int d() {
        if (this.a.size() > 0) {
            return this.a(this.a.size() - 1) + ((f)this.a.lastElement()).b();
        }
        return 0;
    }

    private int e() {
        int n2 = this.b(5);
        if (this.d() > n2) {
            return n2 - this.b(6) - this.b(7);
        }
        return n2;
    }

    private int b(int n2) {
        switch (n2) {
            case 1: {
                int n3 = p.c - (p.b() ? p.c() : 0);
                if (this.c(3) == 32) {
                    return Math.min(this.b(2) + this.d() + this.c(17) + this.c(18), n3);
                }
                n2 = this.c(12);
                if (n2 > 0) {
                    return n2;
                }
                return n3;
            }
            case 5: {
                return this.b(4) - this.c(17) - this.c(18);
            }
            case 2: {
                n2 = Math.max(this.a != null ? this.a.c : 0, this.d != -1 ? p.a(this.d, 1) : 0);
                return n2 + this.c(6) + this.c(7);
            }
            case 4: {
                return this.b(1) - this.b(2);
            }
            case 6: {
                return 1 + p.a(151, 1);
            }
            case 7: {
                return 1 + p.a(150, 1);
            }
        }
        return 0;
    }

    /*
     * Unable to fully structure code
     */
    public final void c(int var1_2) {
        var2_3 = this.a();
        switch (var1_2) {
            case 7: {
                var1_2 = 0;
                break;
            }
            case 5: {
                var1_2 = 1;
                break;
            }
            case 6: 
            case 24: {
                var1_2 = 2;
                break;
            }
            case 1: {
                if (this.a.size() == 0) ** GOTO lbl58
                if (this.b == -1) ** GOTO lbl25
                var7_7 = this.b;
                var6_9 = this;
                for (var3_11 = var7_7 - 1; var3_11 >= 0; --var3_11) {
                    if (!((f)var6_9.a.elementAt(var3_11)).a()) continue;
                    v0 = var3_11;
                    ** GOTO lbl22
                }
                v0 = var1_2 = -1;
lbl22:
                // 2 sources

                if (v0 == -1 || (var2_4 = this.a(var1_2) + ((f)this.a.elementAt(var1_2)).b()) >= this.c - this.c(14)) ** GOTO lbl34
                var1_2 = -1;
                ** GOTO lbl34
lbl25:
                // 1 sources

                var1_2 = -1;
                for (var2_5 = 0; var2_5 < this.a.size(); ++var2_5) {
                    var3_12 = (f)this.a.elementAt(var2_5);
                    var4_16 = this.a(var2_5);
                    var5_18 = var4_16 + var3_12.b();
                    if (var4_16 >= this.c) break;
                    if (!var3_12.a() || var5_18 <= this.c - this.c(14)) continue;
                    var1_2 = var2_5;
                }
lbl34:
                // 4 sources

                if (var1_2 != -1) ** GOTO lbl57
                if (this.c(5) != 128 || this.c > 0) ** GOTO lbl50
                var6_9 = this;
                var7_7 = var6_9.d();
                var3_11 = var6_9.e();
                var6_9.c = var7_7 - var3_11;
                if (var7_7 < var6_9.b(5)) {
                    var6_9.c = 0;
                }
                for (var4_16 = var6_9.a.size() - 1; var4_16 > 0; --var4_16) {
                    if (!var6_9.a(var4_16)) {
                        var6_9.a(-1);
                    } else {
                        if (!((f)var6_9.a.elementAt(var4_16)).a()) continue;
                        var6_9.a(var4_16);
                    }
                    ** GOTO lbl58
                }
                return;
lbl50:
                // 1 sources

                this.c -= this.c(14);
                if (this.c < 0) {
                    this.c = 0;
                }
                if (this.b != -1 && !this.a(this.b)) {
                    this.a(-1);
                    return;
                }
                ** GOTO lbl58
lbl57:
                // 1 sources

                this.a(var1_2);
lbl58:
                // 5 sources

                return;
            }
            case 2: {
                if (this.a.size() == 0) ** GOTO lbl107
                var1_2 = this.d();
                var2_6 = this.e();
                if (this.b == -1) ** GOTO lbl74
                var4_17 = this.b;
                var3_13 = this;
                for (var5_19 = var4_17 + 1; var5_19 < var3_13.a.size(); ++var5_19) {
                    if (!((f)var3_13.a.elementAt(var5_19)).a()) continue;
                    v1 = var5_19;
                    ** GOTO lbl71
                }
                v1 = var3_14 = -1;
lbl71:
                // 2 sources

                if (v1 == -1 || (var4_17 = this.a(var3_14)) <= this.c + var2_6 + this.c(14)) ** GOTO lbl83
                var3_14 = -1;
                ** GOTO lbl83
lbl74:
                // 1 sources

                var3_14 = -1;
                for (var4_17 = this.a.size() - 1; var4_17 >= 0; --var4_17) {
                    var5_20 = (f)this.a.elementAt(var4_17);
                    var6_10 = this.a(var4_17);
                    var7_8 = var6_10 + var5_20.b();
                    if (var7_8 < this.c + var2_6) break;
                    if (!var5_20.a() || var7_8 > this.c + var2_6 + this.c(14)) continue;
                    var3_14 = var4_17;
                }
lbl83:
                // 4 sources

                if (var3_14 != -1) ** GOTO lbl106
                if (this.c(5) != 128 || this.c < var1_2 - var2_6) ** GOTO lbl96
                var3_15 = this;
                this.c = 0;
                for (var4_17 = 0; var4_17 < var3_15.a.size() - 1; ++var4_17) {
                    var5_21 = (f)var3_15.a.elementAt(var4_17);
                    if (!var3_15.a(var4_17)) {
                        var3_15.a(-1);
                    } else {
                        if (!var5_21.a()) continue;
                        var3_15.a(var4_17);
                    }
                    ** GOTO lbl107
                }
                return;
lbl96:
                // 1 sources

                this.c += this.c(14);
                if (var1_2 <= var2_6) {
                    this.c = 0;
                } else if (this.c > var1_2 - var2_6) {
                    this.c = var1_2 - var2_6;
                }
                if (this.b != -1 && !this.a(this.b)) {
                    this.a(-1);
                    return;
                }
                ** GOTO lbl107
lbl106:
                // 1 sources

                this.a(var3_14);
lbl107:
                // 5 sources

                return;
            }
            default: {
                return;
            }
        }
        if ((this.c[var1_2] & 65536) != 0) {
            var0_1 = (short)this.c[var1_2];
            if (var0_1 == -2) {
                if (var2_3 != null) {
                    p.a.d(var2_3.a);
                    return;
                }
            } else {
                p.a.d(var0_1);
            }
        }
    }

    public final void c() {
        if (this.a != null) {
            int n2 = this.d != -1 ? p.a(this.d, 0) : 0;
            int n3 = this.c() - (this.c(8) << 1);
            n2 = this.c(1) != 8 ? n3 - n2 : n3;
            if (n2 < this.a.b) {
                if (this.e >= 3000) {
                    if (this.g > 0) {
                        this.f = (this.e - 3000) * 20 / 1000;
                        if (this.f > this.a.b - n2) {
                            this.f = this.a.b - n2;
                            this.e = 0;
                            this.g = -1;
                        }
                    } else {
                        this.f = this.a.b - n2 - (this.e - 3000) * 20 / 1000;
                        if (this.f <= 0) {
                            this.f = 0;
                            this.e = 0;
                            this.g = 1;
                        }
                    }
                }
                this.e += p.a;
            }
        }
    }

    public final void d() {
        int n2;
        int n3;
        int n4;
        int n5;
        int n6;
        int n7;
        int n8;
        int n9;
        int n10;
        int n11;
        Graphics graphics = p.a();
        int n12 = this.b(2);
        h h2 = this;
        int n13 = h2.c(4) == 64 ? (p.b - h2.c() >> 1) + (p.b() == 3 ? p.d() : 0) : h2.c(9) + (p.b() == 3 ? p.d() : 0);
        if (n.a(this, 1, n13, n11 = (h2 = this).c(4) == 64 ? p.c - (p.b() ? p.c() : 0) - h2.b(1) >> 1 : h2.c(10) + (p.b() == 1 ? p.c() : 0), n10 = this.c(), n9 = this.b(1))) {
            graphics.setColor(this.c(22));
            graphics.fillRect(n13, n11, n10, n9);
            graphics.setColor(this.c(23));
            graphics.drawRect(n13, n11, n10 - 1, n9 - 1);
        }
        if (n12 > 0) {
            graphics.setClip(n13, n11, n10, n12);
            if (n.a(this, 2, n13, n11, n10, n12)) {
                graphics.setColor(this.c(20));
                graphics.fillRect(n13, n11, n10, n12);
            }
            graphics.setClip(n13, n11, n10, n12);
            if (n.a(this, 3, n13, n11, n10, n12)) {
                graphics.setClip(n13, n11, n10, n12);
                n8 = this.c(8);
                n7 = this.d != -1 ? p.a(this.d, 0) : 0;
                n6 = this.a != null ? this.a.b : 0;
                n5 = n10 - (n8 << 1);
                n4 = this.c(1) != 8 ? n5 - n7 : n5;
                if (this.d != -1) {
                    n3 = n8;
                    switch (this.c(1)) {
                        case 8: {
                            n3 = n8 + (n5 - n7 >> 1);
                            break;
                        }
                        case 4: {
                            n3 = n8 + (n5 - n7);
                        }
                    }
                    p.a(n13 + n3, n11 + this.c(6), this.d, 20);
                }
                if (this.a != null) {
                    n3 = n8;
                    switch (this.c(1)) {
                        case 8: {
                            n3 = n8 + (Math.max(n5 - n6, 0) >> 1);
                            break;
                        }
                        case 4: {
                            n3 = n8 + Math.max(n4 - n6, 0);
                            break;
                        }
                        case 0: {
                            n3 = n8 + n7;
                        }
                    }
                    n2 = 0;
                    if (n10 < p.b) {
                        n2 = p.b - n10 - (p.a() ? p.d() : 0) >> 1;
                    }
                    graphics.setClip(n3 - 1 + n2, n11 - 1, n4 + 2, n12 + 2);
                    this.a.a(n13 + n3 - this.f, n11 + this.c(6), this.c(24), this.c(25));
                }
            }
        }
        if (this.c(2) == 16) {
            n8 = p.c();
            n7 = p.d();
            graphics.setClip(0, 0, p.b, p.c);
            graphics.setColor(this.c(21));
            switch (p.b()) {
                case 2: {
                    graphics.fillRect(0, p.c - n8, p.b, n8);
                    break;
                }
                case 1: {
                    graphics.fillRect(0, 0, p.b, n8);
                    break;
                }
                case 3: {
                    graphics.fillRect(0, 0, n7, p.c);
                    break;
                }
                case 4: {
                    graphics.fillRect(p.b - n7, 0, p.b, p.c);
                }
            }
        }
        graphics.setClip(n13, n11 + n12, n10, n9 - n12);
        n.a(this, 4, n13, n11 + n12, n10, n9 - n12);
        n8 = n13 + this.c(15);
        n7 = n11 + n12 + this.c(17);
        n6 = this.b();
        n5 = this.e();
        n4 = this.d();
        n3 = this.b(5);
        graphics.setClip(n8, n7, n6, n3);
        n.a(this, 5, n8, n7, n6, n3);
        graphics.setClip(n8, n7, n6, n3);
        if (this.c > 0) {
            graphics.setClip(n8, n7, n6, this.b(6));
            if (n.a(this, 6, n8, n7, n6, this.b(6))) {
                p.a(n8 + p.a(151, 2) + (n6 - p.a(151, 0) >> 1), n7 + p.a(151, 3), 151);
            }
        }
        if (n5 < n4) {
            n7 += this.b(6);
        }
        if (this.c < n4 - n5) {
            graphics.setClip(n8, n7 + n5, n6, this.b(7));
            if (n.a(this, 7, n8, n7 + n5, n6, this.b(7))) {
                p.a(n8 + p.a(150, 2) + (n6 - p.a(150, 0) >> 1), n7 + n5 + 1 + p.a(150, 3), 150);
            }
        }
        h h3 = this;
        n9 = h3.b;
        if (n9 != -1 && ((f)this.a.elementAt(n9)).a(7) == 2048) {
            f f2 = (f)this.a.elementAt(n9);
            n2 = n8 + h.a(f2, n6);
            int n14 = n7 + this.a(n9) - this.c;
            n13 = f2.a();
            n11 = f2.b();
            graphics.setClip(n2, n14, n13, n11);
            if (n.a(this, 9, n2, n14, n13, n11)) {
                graphics.setColor(f2.a(21));
                graphics.fillRect(n2, n14, n13, n11);
                graphics.setColor(f2.a(22));
                graphics.drawRect(n2, n14, n13 - 1, n11 - 1);
            }
        }
        for (n4 = 0; n4 < this.a.size(); ++n4) {
            f f3 = (f)this.a.elementAt(n4);
            int n15 = this.a(n4);
            if (n15 > this.c + n5) break;
            int n16 = f3.b();
            if (n15 + n16 < this.c) continue;
            n2 = n8 + h.a(f3, n6);
            n15 = n7 + n15 - this.c;
            n13 = f3.a();
            graphics.setClip(n8, n7, n6, n5);
            p.a(n2, n15, n13, n16, e);
            if (!n.a(this, 8, n2, n15, n13, n16)) continue;
            boolean bl = n4 == this.b;
            n13 = n15;
            n15 = n2;
            f f4 = f3;
            b b2 = f4.a;
            if (b2 == null) continue;
            b2.a(n15 + f4.a(13), n13 + f4.a(11), f4.a(f4.a ? (bl ? 18 : 15) : (bl ? 20 : 19)), f4.a(16));
        }
        if (n9 != -1 && ((f)this.a.elementAt(n9)).a(7) == 2048) {
            f f5 = (f)this.a.elementAt(n9);
            n2 = n8 + h.a(f5, n6);
            int n17 = n7 + this.a(n9) - this.c;
            n13 = f5.a();
            int n18 = f5.b();
            graphics.setClip(n2, n17, n13, n18);
            n.a(this, 10, n2, n17, n13, n18);
        }
    }

    public final void d(int n2) {
        Object object = (byte[])p.a(n2);
        if (object == null) {
            return;
        }
        try {
            object = new DataInputStream(new ByteArrayInputStream((byte[])object));
            this.a = h.a((DataInputStream)object, f, 15, 22);
            this.b = h.a((DataInputStream)object, h, 20, 26);
            return;
        }
        catch (Exception exception) {
            return;
        }
    }

    private int c(int n2) {
        return h.a(n2, this.b, 2);
    }

    public final void a(int n2, int n3) {
        this.a = h.a(n2, n3, this.a, 1);
    }

    public final void b(int n2, int n3) {
        this.b = h.a(n2, n3, this.b, 2);
    }

    private static int[] a(DataInputStream dataInputStream, int[] nArray, int n2, int n3) {
        int n4 = dataInputStream.readInt();
        int n5 = nArray.length;
        if (n4 == 0) {
            return null;
        }
        int n6 = 0;
        int n7 = n5;
        while (n4 >>> n7 - 1 != 0) {
            n6 = n7 - n5;
            ++n7;
        }
        int[] nArray2 = new int[n6 + 2];
        int[] nArray3 = nArray2;
        nArray2[0] = n4;
        nArray3[1] = 0;
        if ((n4 & -1 >>> 32 - n5) != 0) {
            nArray3[1] = dataInputStream.readInt();
        }
        for (n4 = 0; n4 < n6; ++n4) {
            if ((nArray3[0] & 1 << n4 + n5) == 0) continue;
            nArray3[n4 + 2] = dataInputStream.readShort();
        }
        for (n4 = n2 + 2 - n5; n4 <= n3 + 2 - n5 && n4 < nArray3.length; ++n4) {
            int n8 = 0 | (nArray3[n4] & 0x7C00) << 9;
            n8 |= (nArray3[n4] & 0x3E0) << 6;
            nArray3[n4] = n8 |= (nArray3[n4] & 0x1F) << 3;
        }
        return nArray3;
    }

    public static int a(int n2, int[] nArray, int n3) {
        int[] nArray2;
        int[] nArray3;
        if (n3 == 1) {
            nArray3 = g;
            nArray2 = f;
        } else {
            nArray3 = i;
            nArray2 = h;
        }
        int n4 = nArray2.length;
        if (!h.a(n2, nArray)) {
            nArray = nArray3;
        }
        if (n2 < n4) {
            return nArray[1] & nArray2[n2];
        }
        return nArray[n2 + 2 - n4];
    }

    public static boolean a(int n2, int[] nArray) {
        return nArray != null && (nArray[0] & 1 << n2) != 0;
    }

    public static int[] a(int n2, int n3, int[] nArray, int n4) {
        int[] nArray2;
        if (n4 == 1) {
            n4 = 16;
            nArray2 = f;
        } else {
            n4 = 23;
            nArray2 = h;
        }
        if (nArray == null) {
            nArray = new int[n4];
        } else if (nArray.length < n4) {
            int[] nArray3 = new int[n4];
            System.arraycopy(nArray, 0, nArray3, 0, nArray.length);
            nArray = nArray3;
        }
        nArray[0] = nArray[0] | 1 << n2;
        if (n2 < nArray2.length) {
            nArray[1] = nArray[1] & ~nArray2[n2];
            nArray[1] = nArray[1] | n3 & nArray2[n2];
        } else {
            nArray[n2 + 2 - nArray2.length] = n3;
        }
        return nArray;
    }
}

