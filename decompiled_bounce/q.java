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
public final class q
extends k {
    public static int[] a;
    public static int[] b;
    private int[] e;
    private int[] f;
    public int[] c;
    public int[] d;
    private short[] a;
    public int a;
    private int b;
    private int c = (int[])2;
    public short a;

    public final int a(byte[] byArray, int n2) {
        n2 = super.a(byArray, n2);
        this.a = (short)(byArray[n2++] << 8 | byArray[n2++] & 0xFF) + 1;
        this.b = (short)(byArray[n2++] << 8 | byArray[n2++] & 0xFF);
        int n3 = byArray[n2++] & 0xFF;
        int n4 = byArray[n2++] & 0xFF;
        int n5 = byArray[n2++] & 0xFF;
        int n6 = byArray[n2++] & 0xFF;
        this.c = n3 << 24 | n4 << 16 | n5 << 8 | n6;
        this.c = new int[this.a];
        this.d = new int[this.a];
        this.e = new int[this.a];
        this.f = new int[this.a];
        this.a = new short[this.b];
        n3 = byArray[n2++];
        n4 = (short)(byArray[n2++] << 8 | byArray[n2++] & 0xFF);
        n2 = q.a(this.c, this.a - 1, n4, byArray, n2, n3);
        n4 = (short)(byArray[n2++] << 8 | byArray[n2++] & 0xFF);
        n2 = q.a(this.d, this.a - 1, n4, byArray, n2, n3);
        n3 = byArray[n2++];
        n2 = q.a(this.a, this.b, 0, 1, byArray, n2, n3);
        this.c[this.a - 1] = this.c[0];
        this.d[this.a - 1] = this.d[0];
        this.a = (short)(byArray[n2++] << 8 | byArray[n2++] & 0xFF);
        this.a();
        return n2;
    }

    public final void a() {
        int n2;
        super.a();
        for (n2 = 0; n2 < this.c.length; ++n2) {
            if (this.c[n2] < this.e) {
                this.e = (int[])this.c[n2];
            }
            if (this.c[n2] <= this.g) continue;
            this.g = this.c[n2];
        }
        for (n2 = 0; n2 < this.d.length; ++n2) {
            if (this.d[n2] < this.f) {
                this.f = (int[])this.d[n2];
            }
            if (this.d[n2] <= this.h) continue;
            this.h = this.d[n2];
        }
    }

    public final void a(Graphics graphics, DirectGraphics object, d d2) {
        int n2;
        int n3;
        d d3 = d2;
        object = this;
        if (((q)object).g) {
            ((k)object).a(k.a);
            n3 = d3.c;
            n2 = d3.f;
            d3.c = 0;
            d3.f = 0;
            d.a(d3, k.a, d.a);
            d3.c = n3;
            d3.f = n2;
            for (int i2 = 0; i2 < ((q)object).a; ++i2) {
                d.a.a(((q)object).c[i2], ((q)object).d[i2]);
                ((q)object).e[i2] = d.g >> 16;
                ((q)object).f[i2] = d.h >> 16;
            }
            ((q)object).g = false;
        }
        int n4 = p.b - 1;
        int n5 = p.c - 1;
        n3 = d2.c >> 16;
        int n6 = d2.f >> 16;
        graphics.setColor(n.c(this.c));
        for (n2 = 0; n2 < this.b; n2 += 3) {
            int n7 = this.a[n2];
            int n8 = this.a[n2 + 1];
            int n9 = this.a[n2 + 2];
            int n10 = this.e[n7] + n3;
            int n11 = this.e[n8] + n3;
            int n12 = this.e[n9] + n3;
            n7 = this.f[n7] + n6;
            n8 = this.f[n8] + n6;
            n9 = this.f[n9] + n6;
            if (n10 < 0 && n11 < 0 && n12 < 0 || n7 < 0 && n8 < 0 && n9 < 0 || n10 > n4 && n11 > n4 && n12 > n4 || n7 > n5 && n8 > n5 && n9 > n5) continue;
            graphics.fillTriangle(n10, n7, n11, n8, n12, n9);
        }
    }
}

