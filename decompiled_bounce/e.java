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
public final class e
extends k {
    short a;
    c a;
    int a;
    private int c = 8;
    int b;
    float a;
    byte a;
    boolean a;

    public final int a(byte[] byArray, int n2) {
        n2 = super.a(byArray, n2);
        this.a = (short)(byArray[n2++] << 8 | byArray[n2++] & 0xFF);
        this.a = byArray[n2++];
        this.a = 0;
        this.b = 0;
        this.c = 0;
        int n3 = n.b(n.f);
        if (n3 == 0) {
            this.a = (short)497;
        }
        if (n3 == 1) {
            this.a = (short)509;
        }
        if (n3 == 2) {
            this.a = (short)490;
        }
        return n2;
    }

    public final void a() {
        this.e = -4587520;
        this.g = 0x460000;
        this.f = 0;
        this.h = 0x5F0000;
    }

    public final void d() {
        super.d();
        if (this.a < this.b) {
            int n2 = p.a;
            this.a += n2;
            if (this.a >= this.b - this.b / p.b(this.a)) {
                if (!this.a) {
                    this.a = true;
                    this.a.a = true;
                    this.a.b = this.a;
                    this.a.j = 0.0f;
                    this.a.k = 0.0f;
                    this.a.c = true;
                    if (this.a.equals(n.a)) {
                        g.a[1] = 0;
                    }
                }
                n2 = p.b(this.a);
                this.c = this.a * n2 / this.b;
            } else {
                n2 = p.b(this.a);
                this.c = this.a * n2 / this.b;
                this.b();
            }
            if (this.a >= this.b) {
                this.a = 0;
                this.b = 0;
                this.c = 0;
            }
        }
    }

    public final void b() {
        this.a(k.a);
        this.a.c.f = k.a.f - (65536 * (short)p.b(p.c(this.a, this.c), 0) / k.g.a << 16);
    }

    public final void a(Graphics graphics, DirectGraphics directGraphics, d d2) {
        this.a(k.a);
        d.a(d2, k.a, d.a);
        int n2 = d.a.c >> 16;
        int n3 = d.a.f >> 16;
        p.b(n2, n3, this.a, this.c);
    }
}

