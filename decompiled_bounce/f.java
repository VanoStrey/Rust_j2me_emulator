/*
 * Decompiled with CFR 0.152.
 */
/*
 * Duplicate member names - consider using --renamedupmembers true
 */
public final class f {
    public h a;
    public boolean a;
    public int a;
    public b a;
    private int[] a;
    public String a = null;

    public f() {
    }

    public f(String string, int n2, h h2, int n3) {
        this.a = h2;
        this.a = n3;
        this.a(string, n2);
    }

    public final int a() {
        if (this.a(5) == 256) {
            b b2 = this.a;
            if (b2 != null) {
                if (b2.a == null || b2.a.length() < 1) {
                    return b2.b - b.a + this.a(13) + this.a(14);
                }
                return b2.b + this.a(13) + this.a(14);
            }
            if (this.a != null) {
                return this.a.b();
            }
        } else if (this.a(8) == 4096) {
            if (this.a != null) {
                return this.a.b();
            }
        } else {
            return this.a(10);
        }
        return 0;
    }

    public final int b() {
        b b2 = this.a;
        return (b2 != null ? b2.c : 0) + this.a(11) + this.a(12);
    }

    public final boolean a() {
        return this.a != -1;
    }

    public final void a(int n2, int n3) {
        this.a = h.a(n2, n3, this.a, 1);
    }

    public final int a(int n2) {
        if (h.a(n2, this.a) || this.a == null) {
            return h.a(n2, this.a, 1);
        }
        return h.a(n2, this.a.a, 1);
    }

    public final void a(String string, int n2) {
        if (string == null) {
            string = "";
        }
        b b2 = this.a;
        this.a = null;
        int n3 = this.a();
        this.a = b2;
        n3 -= this.a(13) + this.a(14);
        int n4 = 0;
        for (int i2 = 0; i2 < 9; ++i2) {
            n4 |= this.a(i2);
        }
        this.a = new b(string, n3, this.a(9), n4, n2);
    }
}

