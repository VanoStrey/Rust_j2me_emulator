/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  com.nokia.mid.ui.DirectGraphics
 *  javax.microedition.lcdui.Graphics
 */
import com.nokia.mid.ui.DirectGraphics;
import javax.microedition.lcdui.Graphics;

public final class a
extends k {
    public a() {
        this.c = (byte)9;
    }

    public final int a(byte[] byArray, int n2) {
        n2 = super.a(byArray, n2);
        return n2;
    }

    public final void a() {
        this.e = -2949120;
        this.g = 0x2D0000;
        this.f = -2949120;
        this.h = 0x2D0000;
    }

    public final void a(Graphics graphics, DirectGraphics directGraphics, d d2) {
        this.a(k.a);
        d.a(d2, k.a, d.a);
        int n2 = d.a.c >> 16;
        int n3 = d.a.f >> 16;
        p.a(n2, n3, 208);
    }
}

