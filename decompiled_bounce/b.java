/*
 * Decompiled with CFR 0.152.
 */
/*
 * Duplicate member names - consider using --renamedupmembers true
 */
public final class b {
    public static final int a;
    private int d;
    private int e;
    private int f;
    private int g;
    private int h = 0;
    private int i;
    public int b;
    public int c;
    private int j;
    String a;
    private String[] a;
    private int k;
    private int l;
    private boolean a;

    public b(String string, int n2, int n3, int n4, int n5) {
        this.e = n4;
        this.a = string;
        this.f = n3;
        switch (this.e & 3) {
            case 2: {
                this.g = 3;
                break;
            }
            case 1: {
                this.g = 2;
                break;
            }
            default: {
                this.g = 1;
            }
        }
        if (n5 >= 0) {
            this.d = n5;
            this.l = p.a(n5, 0) + a;
            this.k = p.a(n5, 1);
        } else {
            this.d = -1;
            this.l = 0;
            this.k = 0;
        }
        this.a = n2 > this.l && (this.e & 0x80) == 128 && (this.e & 0x30) != 32;
        n3 = 0;
        if ((this.e & 0x100) == 256) {
            n3 = 1;
        }
        this.a = string != null ? string : "";
        this.a = p.a(this.a, n2, this.f, this.a, this.l, this.k, n3 != 0);
        this.a(this.a[0]);
    }

    private void a(String string) {
        int[] nArray = new int[4];
        if (string != null) {
            int n2 = 0;
            for (int i2 = 0; i2 < string.length(); ++i2) {
                if (string.charAt(i2) != ' ') continue;
                nArray[n2] = i2;
                ++n2;
            }
            try {
                this.j = Integer.valueOf(string.substring(0, nArray[0]));
            }
            catch (Exception exception) {}
            try {
                this.h = Integer.valueOf(string.substring(nArray[0] + 1, nArray[1]));
            }
            catch (Exception exception) {
                this.h = 0;
            }
            try {
                this.i = Integer.valueOf(string.substring(nArray[1] + 1, nArray[2]));
            }
            catch (Exception exception) {
                this.i = 0;
            }
            try {
                this.b = Integer.valueOf(string.substring(nArray[2] + 1, nArray[3]));
            }
            catch (Exception exception) {
                this.b = 0;
            }
            try {
                this.c = Integer.valueOf(string.substring(nArray[3] + 1, string.length()));
            }
            catch (NumberFormatException numberFormatException) {
                this.c = 0;
                return;
            }
        }
        this.j = 0;
        this.h = 0;
        this.i = 0;
        this.b = 0;
        this.c = 0;
    }

    public final void a(int n2, int n3, int n4, int n5) {
        int n6;
        boolean bl = true;
        int n7 = this.j;
        bl = false;
        b b2 = this;
        p.a(b2.f, b2.g);
        p.b(0, n4);
        p.b(1, n5);
        if (n7 > b2.j) {
            n7 = b2.j;
        }
        if ((b2.e & 0x30) == 32) {
            n4 = b2.b / 2;
            n5 = 17;
        } else {
            n6 = (b2.e & 0xC) == 8 ? (b2.b - b2.i) / 2 : 0;
            if ((b2.e & 0x30) == 0) {
                n4 = n6;
                n5 = 20;
            } else {
                n4 = b2.b - n6;
                n5 = 24;
            }
        }
        n6 = (b2.e & 0x30) == 0 ? 1 : 0;
        int n8 = 0;
        int n9 = 0;
        int n10 = 0;
        int n11 = 0;
        switch (b2.e & 0xC) {
            case 0: {
                n9 = 20;
                n11 = 0;
                n8 = 20;
                if (n6 != 0) {
                    n10 = b2.l;
                    break;
                }
                n10 = 0;
                break;
            }
            case 4: {
                n9 = 24;
                n11 = b2.b;
                n8 = 24;
                if (n6 != 0) {
                    n10 = b2.b;
                    break;
                }
                n10 = b2.b - b2.l;
                break;
            }
            case 8: {
                n9 = 17;
                n11 = b2.b / 2;
                if (n6 != 0) {
                    n8 = 20;
                    n10 = (b2.b - b2.i) / 2 + b2.l;
                    break;
                }
                n8 = 24;
                n10 = b2.b - (b2.b - b2.i) / 2 - b2.l;
            }
        }
        if (b2.d >= 0) {
            p.a(n2 + n4, n3, b2.d, n5);
        }
        if (!b2.a) {
            n3 += b2.k;
        }
        n4 = 0 * p.a(b2.f);
        for (n5 = 0; n5 < n7; ++n5) {
            if (b2.a[n5 + 1] == null) continue;
            if (n5 < b2.h) {
                p.a(b2.a[n5 + 1], 0, b2.a[n5 + 1].length(), n2 + n10, n3 + n4, n8);
            } else {
                p.a(b2.a[n5 + 1], 0, b2.a[n5 + 1].length(), n2 + n11, n3 + n4, n9);
            }
            n4 += p.a(b2.f);
        }
    }

    static {
        char[] cArray = new char[]{'\n', ' ', '-'};
        a = p.b >>> 5;
    }
}

