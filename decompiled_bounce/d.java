/*
 * Decompiled with CFR 0.152.
 */
/*
 * Duplicate member names - consider using --renamedupmembers true
 */
public final class d {
    public int a = 65536;
    public int b;
    public int c;
    public int d;
    public int e = 65536;
    public int f;
    public static int g;
    public static int h;
    public static d a;

    public final void a(int n2, int n3) {
        g = (int)((long)this.a * (long)n2 + (long)this.b * (long)n3 >> 16) + this.c;
        h = (int)((long)this.d * (long)n2 + (long)this.e * (long)n3 >> 16) + this.f;
    }

    public final void b(int n2, int n3) {
        g = (int)((long)this.a * (long)n2 + (long)this.b * (long)n3 >> 16);
        h = (int)((long)this.d * (long)n2 + (long)this.e * (long)n3 >> 16);
    }

    public final void a(d d2) {
        this.a = d2.a;
        this.b = d2.b;
        this.c = d2.c;
        this.d = d2.d;
        this.e = d2.e;
        this.f = d2.f;
    }

    public final void b(d d2) {
        int n2 = (int)((long)this.a * (long)d2.a + (long)this.b * (long)d2.d >> 16);
        int n3 = (int)((long)this.a * (long)d2.b + (long)this.b * (long)d2.e >> 16);
        int n4 = (int)(((long)this.a * (long)d2.c + (long)this.b * (long)d2.f >> 16) + (long)this.c);
        int n5 = (int)((long)this.d * (long)d2.a + (long)this.e * (long)d2.d >> 16);
        int n6 = (int)((long)this.d * (long)d2.b + (long)this.e * (long)d2.e >> 16);
        int n7 = (int)(((long)this.d * (long)d2.c + (long)this.e * (long)d2.f >> 16) + (long)this.f);
        this.a = n2;
        this.b = n3;
        this.c = n4;
        this.d = n5;
        this.e = n6;
        this.f = n7;
    }

    public static void a(d d2, d d3, d d4) {
        int n2 = (int)((long)d2.a * (long)d3.a + (long)d2.b * (long)d3.d >> 16);
        int n3 = (int)((long)d2.a * (long)d3.b + (long)d2.b * (long)d3.e >> 16);
        int n4 = (int)(((long)d2.a * (long)d3.c + (long)d2.b * (long)d3.f >> 16) + (long)d2.c);
        int n5 = (int)((long)d2.d * (long)d3.a + (long)d2.e * (long)d3.d >> 16);
        int n6 = (int)((long)d2.d * (long)d3.b + (long)d2.e * (long)d3.e >> 16);
        int n7 = (int)(((long)d2.d * (long)d3.c + (long)d2.e * (long)d3.f >> 16) + (long)d2.f);
        d4.a = n2;
        d4.b = n3;
        d4.c = n4;
        d4.d = n5;
        d4.e = n6;
        d4.f = n7;
    }

    public final void c(d d2) {
        long l2 = (long)this.a * (long)this.e - (long)this.b * (long)this.d >> 16;
        if (l2 != 0L) {
            d2.a = (int)(((long)this.e << 16) / l2);
            d2.b = (int)(((long)this.d << 16) / l2);
            d2.d = (int)(((long)this.b << 16) / l2);
            d2.e = (int)(((long)this.a << 16) / l2);
            d2.c = -((int)((long)this.c * (long)d2.a + (long)this.f * (long)d2.b >> 16));
            d2.f = -((int)((long)this.c * (long)d2.d + (long)this.f * (long)d2.e >> 16));
            return;
        }
        throw new ArithmeticException("Non-invertible matrix.");
    }

    public final void a(float f2) {
        int n2 = (int)((float)Math.cos(f2) * 65536.0f);
        int n3 = (int)((float)Math.sin(f2) * 65536.0f);
        this.a = n2;
        this.b = -n3;
        this.d = n3;
        this.e = n2;
    }

    public final String toString() {
        return new String(this.a + " " + this.b + " " + this.c + "\n" + this.d + " " + this.e + " " + this.f);
    }

    static {
        a = new d();
    }
}

