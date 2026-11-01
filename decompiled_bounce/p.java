/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  javax.microedition.lcdui.Command
 *  javax.microedition.lcdui.CommandListener
 *  javax.microedition.lcdui.Displayable
 *  javax.microedition.lcdui.Font
 *  javax.microedition.lcdui.Graphics
 *  javax.microedition.lcdui.Image
 *  javax.microedition.lcdui.game.GameCanvas
 *  javax.microedition.media.Manager
 *  javax.microedition.media.Player
 *  javax.microedition.media.control.VolumeControl
 *  javax.microedition.midlet.MIDlet
 *  javax.microedition.rms.RecordStore
 *  javax.microedition.rms.RecordStoreException
 */
import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.DataInputStream;
import java.io.DataOutputStream;
import java.io.FilterInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.Vector;
import javax.microedition.lcdui.Command;
import javax.microedition.lcdui.CommandListener;
import javax.microedition.lcdui.Displayable;
import javax.microedition.lcdui.Font;
import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.Image;
import javax.microedition.lcdui.game.GameCanvas;
import javax.microedition.media.Manager;
import javax.microedition.media.Player;
import javax.microedition.media.control.VolumeControl;
import javax.microedition.midlet.MIDlet;
import javax.microedition.rms.RecordStore;
import javax.microedition.rms.RecordStoreException;

/*
 * Duplicate member names - consider using --renamedupmembers true
 */
public final class p
extends GameCanvas
implements j,
Runnable,
CommandListener {
    private static p a = null;
    public static boolean a = false;
    private static boolean b = false;
    private static boolean c = true;
    private static boolean d = false;
    private static int d = 500;
    private static Object a = new Object();
    private static Object b = new Object();
    public static n a;
    private static int[] a;
    private static int e;
    private static int[] b;
    private static int f;
    private static boolean e;
    private static boolean f;
    private static boolean g;
    private static boolean h;
    private static int g;
    private static long a;
    private static boolean i;
    private static int h;
    private static long b;
    public static int a;
    private static int i;
    private static int j;
    private static String[] a;
    private static String[] b;
    private static int[] c;
    private static char[] a;
    public static int b;
    public static int c;
    private static Graphics a;
    private static Graphics b;
    private static Font[] a;
    private static int[] d;
    private static int k;
    private static int l;
    private static int m;
    private static Vector a;
    private static Vector b;
    private static j[] a;
    private static Object[] a;
    private static boolean[] a;
    private static String[] c;
    private static int[] e;
    private static short[][] a;
    private static short[][] b;
    private static short[] a;
    private static Image[] a;
    private static byte[] a;
    private static final short[] b;
    private static int n;
    private static String[] d;
    private static int o;
    private static int p;
    private static boolean j;
    private static boolean k;
    private static Player a;
    private static Player b;
    private static int q;
    private static int r;
    public static MIDlet a;
    private static int s;
    private static int t;
    private static int u;
    private static int v;
    private static int[] f;
    private static int w;
    private static boolean l;
    private static long c;
    private static int x;
    private static int y;
    private static long d;
    private static boolean m;
    private static int z;
    private static char a;
    private static char[][] a;
    private static char[][] b;
    private static boolean n;

    public p() {
        super(false);
    }

    private static void i(int n2) {
        if (e == 20) {
            return;
        }
        p.a[p.e] = n2;
        ++e;
    }

    public static void a(int n2) {
        if (f == 40) {
            throw new RuntimeException();
        }
        p.b[p.f] = n2;
        ++f;
    }

    public static void a() {
        h = true;
    }

    public static void b(int n2) {
        i = 2;
    }

    public static int a() {
        return i;
    }

    public static void c(int n2) {
        h = n2;
    }

    public static void a(int n2, String string, int n3) {
        p.a[n2] = string;
        p.c[n2] = n3;
    }

    public static void b() {
        for (int i2 = 0; i2 < 3; ++i2) {
            p.a(i2, null, -1);
        }
    }

    public static int b() {
        if ((p.c(0) & 0x72) == (p.c(1) & 0x72)) {
            if ((p.c(0) & 0x20) == 32) {
                return 2;
            }
            return 1;
        }
        if ((p.c(0) & 0xD) == (p.c(1) & 0xD)) {
            if ((p.c(0) & 4) == 4) {
                return 3;
            }
            return 4;
        }
        return 0;
    }

    public static int c() {
        if (p.b() != 0) {
            return n.b();
        }
        return 0;
    }

    public static int d() {
        if (p.b() != 0) {
            return -1;
        }
        return 0;
    }

    private static int c(int n2) {
        switch (n2) {
            case 0: {
                return 33;
            }
            case 1: {
                return 40;
            }
            case 2: {
                return 36;
            }
        }
        return -1;
    }

    public static boolean a() {
        return p.b() == 3 || p.b() == 4;
    }

    public static boolean b() {
        return p.b() == 1 || p.b() == 2;
    }

    public static String[] a(String string, int n2, int n3, boolean bl, int n4, int n5, boolean bl2) {
        String[] stringArray = new String[5];
        int n6 = 0;
        int n7 = 0;
        int n8 = n4;
        int n9 = 0;
        int n10 = string.length();
        int n11 = 0;
        while (n11 < n10) {
            int n12;
            int n13;
            boolean bl3 = bl && p.a(n3) * n6 < n5;
            if (bl3) {
                n13 = n2 - n4;
                ++n7;
            } else {
                n13 = n2;
            }
            int n14 = 0;
            int n15 = n11;
            int n16 = n10;
            block3: while (n11 < n10) {
                int n17;
                n12 = n10;
                for (n17 = n11; n17 < n10; ++n17) {
                    int n18;
                    for (n18 = 0; n18 < a.length && string.charAt(n17) != a[n18]; ++n18) {
                    }
                    if (n18 >= a.length) continue;
                    n12 = n17;
                    if (string.charAt(n12) != '-') break;
                    ++n12;
                    break;
                }
                if (p.a(n3, string, n15, n12 - n15) < n13) {
                    n16 = n12;
                    ++n14;
                    if (n16 < n10 && string.charAt(n16) == '\n') break;
                    for (n11 = n12; n11 < n10; ++n11) {
                        for (n17 = 0; n17 < a.length && string.charAt(n11) != a[n17]; ++n17) {
                        }
                        if (n17 >= a.length || string.charAt(n11) == '-' || string.charAt(n11) == '\n') continue block3;
                    }
                    continue;
                }
                if (n14 != 0) break;
                n17 = n15;
                while (p.a(n3, string, n15, n17 - n15 + 1) < n13) {
                    ++n17;
                }
                n16 = n17;
                n11 = n17;
                break;
            }
            n12 = p.a(n3, string, n15, n16 - n15);
            if (bl3) {
                n8 = Math.max(n8, n12 + n4);
                n9 = Math.max(n9, n12 + n4);
            } else {
                n9 = Math.max(n9, n12);
            }
            stringArray = a ? p.a(stringArray, n6 + 1, '\u200f' + string.substring(n15, n16)) : p.a(stringArray, n6 + 1, string.substring(n15, n16));
            ++n6;
            if (n11 < n10 && string.charAt(n11) == '\n') {
                ++n11;
            }
            try {
                Thread.sleep(1L);
            }
            catch (InterruptedException interruptedException) {}
        }
        int n19 = bl2 ? n9 : n2;
        n2 = bl ? Math.max(n6 * p.a(n3), n5) : n6 * p.a(n3) + n5;
        stringArray[0] = "" + n6 + " " + n7 + " " + n8 + " " + n19 + " " + n2;
        return stringArray;
    }

    private static String[] a(String[] stringArray, int n2, String string) {
        if (stringArray.length <= n2) {
            String[] stringArray2 = new String[stringArray.length + 5];
            System.arraycopy(stringArray, 0, stringArray2, 0, stringArray.length);
            stringArray = stringArray2;
        }
        stringArray[n2] = string;
        return stringArray;
    }

    public static int e() {
        return b;
    }

    public static boolean a(int n2) {
        return (v & 1 << n2) != 0;
    }

    public static void d(int n2) {
        t = 0;
        s = 2;
    }

    public static void c() {
        t = 0;
        v = 0;
        u = 0;
        w = 0;
        l = true;
    }

    /*
     * Enabled aggressive block sorting
     * Enabled unnecessary exception pruning
     * Enabled aggressive exception aggregation
     */
    public static void a(String object, byte[] byArray) {
        RecordStore recordStore = null;
        try {
            recordStore = RecordStore.openRecordStore((String)object, (boolean)true);
            if (recordStore.getNumRecords() < 1) {
                recordStore.addRecord(null, 0, 0);
            }
            byte[] byArray2 = new byte[byArray.length + 2];
            object = byArray2;
            byArray2[0] = 1;
            object[1] = 7;
            System.arraycopy(byArray, 0, object, 2, byArray.length);
            recordStore.setRecord(1, (byte[])object, 0, ((Object)object).length);
        }
        catch (RecordStoreException recordStoreException) {
            try {
                if (recordStore == null) return;
                recordStore.closeRecordStore();
                return;
            }
            catch (Exception exception) {
                return;
            }
        }
        catch (Exception exception) {
            try {
                if (recordStore == null) return;
                recordStore.closeRecordStore();
                return;
            }
            catch (Exception exception2) {
                return;
            }
        }
        catch (Throwable throwable) {
            try {
                if (recordStore == null) throw throwable;
                recordStore.closeRecordStore();
                throw throwable;
            }
            catch (Exception exception) {}
            throw throwable;
        }
        try {
            if (recordStore == null) return;
            recordStore.closeRecordStore();
            return;
        }
        catch (Exception exception) {
            return;
        }
    }

    /*
     * Loose catch block
     */
    public static byte[] a(String object) {
        RecordStore recordStore;
        block25: {
            block24: {
                recordStore = null;
                recordStore = RecordStore.openRecordStore((String)object, (boolean)false);
                if (recordStore != null && recordStore.getNumRecords() >= 1) break block24;
                try {
                    if (recordStore != null) {
                        recordStore.closeRecordStore();
                    }
                }
                catch (Exception exception) {}
                return null;
            }
            byte[] byArray = recordStore.getRecord(1);
            object = byArray;
            if (byArray != null && ((Object)object).length >= 2 && object[0] == true && object[1] == 7) break block25;
            try {
                if (recordStore != null) {
                    recordStore.closeRecordStore();
                }
            }
            catch (Exception exception) {}
            return null;
        }
        byte[] byArray = new byte[((Object)object).length - 2];
        System.arraycopy(object, 2, byArray, 0, byArray.length);
        object = byArray;
        try {
            if (recordStore != null) {
                recordStore.closeRecordStore();
            }
        }
        catch (Exception exception) {}
        return object;
        catch (RecordStoreException recordStoreException) {
            try {
                if (recordStore != null) {
                    recordStore.closeRecordStore();
                }
            }
            catch (Exception exception) {}
            return null;
        }
        catch (Exception exception) {
            try {
                if (recordStore != null) {
                    recordStore.closeRecordStore();
                }
            }
            catch (Exception exception2) {}
            return null;
        }
        catch (Throwable throwable) {
            try {
                if (recordStore != null) {
                    recordStore.closeRecordStore();
                }
            }
            catch (Exception exception) {}
            throw throwable;
        }
    }

    public static Graphics a() {
        return b;
    }

    public static void a(Graphics graphics) {
        b = graphics != null ? graphics : a;
    }

    public static boolean a(int n2, int n3, int n4, int n5, int[] nArray) {
        int n6 = b.getClipX();
        int n7 = b.getClipY();
        int n8 = b.getClipWidth();
        int n9 = b.getClipHeight();
        nArray[0] = n6;
        nArray[1] = n7;
        nArray[2] = n8;
        nArray[3] = n9;
        if (n2 < n6) {
            n4 -= n6 - n2;
            n2 = n6;
        }
        if (n3 < n7) {
            n5 -= n7 - n3;
            n3 = n7;
        }
        if (n2 + n4 > n6 + n8) {
            n4 = n6 + n8 - n2;
        }
        if (n3 + n5 > n7 + n9) {
            n5 = n7 + n9 - n3;
        }
        if (n4 > 0 && n5 > 0) {
            b.setClip(n2, n3, n4, n5);
            return true;
        }
        return false;
    }

    public static int a(int n2, String string, int n3, int n4) {
        if (n2 == -1) {
            n2 = 0;
        } else if (n2 == -2) {
            n2 = 1;
        } else if (n2 == -3) {
            n2 = 2;
        } else {
            return -1;
        }
        return a[n2].substringWidth(string, n3, n4);
    }

    public static int a(int n2) {
        if (n2 == -1) {
            n2 = 0;
        } else if (n2 == -2) {
            n2 = 1;
        } else if (n2 == -3) {
            n2 = 2;
        } else {
            return -1;
        }
        n2 = a[n2].getHeight();
        return n2;
    }

    public static void a(int n2, int n3) {
        m = n2;
        l = n3;
        p.d[0] = 0;
    }

    public static int f() {
        return m;
    }

    public static void b(int n2, int n3) {
        p.d[n2] = n3;
    }

    public static void a(String string, int n2, int n3, int n4, int n5, int n6) {
        if (m == -1) {
            n2 = 0;
        } else if (m == -2) {
            n2 = 1;
        } else if (m == -3) {
            n2 = 2;
        } else {
            return;
        }
        if ((n6 & 2) == 2) {
            n6 = n6 & 0xFFFFFFFD | 0x10;
            n5 -= p.a(m) / 2;
        }
        b.setFont(a[n2]);
        if (l != 1) {
            b.setColor(d[1]);
            switch (l) {
                case 3: {
                    n2 = 0;
                    while (n2 < 4) {
                        b.drawSubstring(string, 0, n3, n4 + (n2 & 1) * (n2 - 2), n5 + (++n2 & 1) * (n2 - 2), n6);
                    }
                    break;
                }
                case 2: {
                    b.drawSubstring(string, 0, n3, n4 + 1, n5 + 1, n6);
                }
            }
        }
        b.setColor(d[0]);
        b.drawSubstring(string, 0, n3, n4, n5, n6);
    }

    public static void e(int n2) {
        if (n2 < 0) {
            return;
        }
        Integer n3 = new Integer(n2);
        if (!a.contains(n3)) {
            a.addElement(n3);
        }
        b.removeElement(n3);
    }

    public static void f(int n2) {
        if (n2 < 0) {
            return;
        }
        Integer n3 = new Integer(n2);
        if (!b.contains(n3)) {
            b.addElement(n3);
        }
        a.removeElement(n3);
    }

    public static Object a(int n2) {
        if (n2 < 0) {
            return null;
        }
        if (a != null) {
            return a[n2];
        }
        return null;
    }

    public static boolean b(int n2) {
        if (n2 < 0) {
            return false;
        }
        return a != null && a[n2];
    }

    private static void a(DataInputStream dataInputStream, int n2) {
        int n3 = 0;
        while (n3 < n2) {
            n3 = dataInputStream.skipBytes(n2 - n3);
        }
    }

    private static byte[] a(InputStream inputStream, int n2) {
        int n3;
        if (n2 != -1) {
            byte[] byArray = new byte[n2];
            for (int i2 = 0; i2 < byArray.length; i2 += inputStream.read(byArray, i2, byArray.length - i2)) {
            }
            return byArray;
        }
        ByteArrayOutputStream byteArrayOutputStream = new ByteArrayOutputStream();
        byte[] byArray = new byte[1024];
        while ((n3 = inputStream.read(byArray)) >= 0) {
            byteArrayOutputStream.write(byArray, 0, n3);
        }
        return byteArrayOutputStream.toByteArray();
    }

    private static void e() {
        if (c != null) {
            return;
        }
        try {
            int n2;
            short s2;
            int n3;
            Object object = new DataInputStream(a.getClass().getResourceAsStream("/a"));
            int n4 = ((DataInputStream)object).readShort();
            c = new String[n4];
            e = new int[n4 << 1];
            for (n3 = 0; n3 < n4; ++n3) {
                p.c[n3] = ((DataInputStream)object).readUTF();
                p.e[n3 << 1] = ((DataInputStream)object).readInt();
                p.e[(n3 << 1) + 1] = ((DataInputStream)object).readInt();
            }
            n4 = ((DataInputStream)object).readShort();
            a = new short[n4][];
            a = new Object[n4];
            a = new boolean[n4];
            for (n3 = 0; n3 < n4; ++n3) {
                s2 = ((DataInputStream)object).readByte();
                n2 = ((DataInputStream)object).readByte();
                p.a[n3] = new short[n2 + 2];
                p.a[n3][0] = s2;
                p.a[n3][1] = ((DataInputStream)object).readShort();
                for (int i2 = 0; i2 < n2; ++i2) {
                    p.a[n3][i2 + 2] = ((DataInputStream)object).readShort();
                }
            }
            n4 = ((DataInputStream)object).readShort();
            short[] sArray = new short[n4];
            short[] sArray2 = new short[n4];
            for (n3 = 0; n3 < n4; ++n3) {
                sArray[n3] = ((DataInputStream)object).readShort();
                sArray2[n3] = ((DataInputStream)object).readShort();
            }
            ((FilterInputStream)object).close();
            for (n3 = 0; n3 < sArray2.length; ++n3) {
                s2 = sArray2[n3];
                object = p.a(s2);
                if (object == null) continue;
                for (n2 = 0; n2 < a.length && !a[n2].a((DataInputStream)object, (int)sArray[n3]); ++n2) {
                }
                ((FilterInputStream)object).close();
            }
            a[0].a(null, -1);
            return;
        }
        catch (IOException iOException) {
            return;
        }
    }

    private static void f() {
        int n2;
        int n3;
        Object object;
        int n4;
        int n5;
        int n6;
        Vector<int[]> vector = new Vector<int[]>();
        for (n6 = 0; n6 < a.size(); ++n6) {
            int n7 = (Integer)a.elementAt(n6);
            if (a[n7]) continue;
            int n8 = 0;
            n5 = -1;
            while (n5 < a[n7].length - 2) {
                n4 = n5 == -1 ? a[n7][1] : a[n7][n5 + 2];
                int n9 = vector.size();
                for (int i2 = n8; i2 < vector.size(); ++i2) {
                    object = (int[])vector.elementAt(i2);
                    n3 = c[n4].equals(c[object[2]]) ? 1 : 0;
                    int n10 = n2 = i2 == vector.size() - 1 || !c[n4].equals(c[((int[])vector.elementAt(i2 + 1))[2]]) ? 1 : 0;
                    if (n3 != 0 && e[n4 << 1] < e[object[2] << 1]) {
                        n9 = i2;
                        break;
                    }
                    if (n3 == 0 || n2 == 0) continue;
                    n9 = i2 + 1;
                    break;
                }
                if (n5 == -1) {
                    n8 = n9 + 1;
                }
                int[] nArray = new int[3];
                int[] nArray2 = nArray;
                nArray[0] = n7;
                nArray2[1] = n5++;
                nArray2[2] = n4;
                vector.insertElementAt(nArray2, n9);
            }
        }
        try {
            Object object2 = null;
            Object object3 = null;
            n5 = 0;
            for (n4 = 0; n4 < vector.size(); ++n4) {
                int n11;
                Object object4 = (int[])vector.elementAt(n4);
                int n12 = object4[0];
                n6 = object4[2];
                object = c[n6];
                n3 = e[n6 << 1];
                n2 = e[(n6 << 1) + 1];
                if (object2 != null && ((String)object).equals(object3) && n5 <= n3) {
                    p.a((DataInputStream)object2, n3 - n5);
                } else {
                    if (object2 != null) {
                        ((FilterInputStream)object2).close();
                        object2 = null;
                    }
                    if (n3 != -1 && n2 != 0) {
                        object2 = p.a(n6);
                    }
                }
                n6 = 0;
                if (object4[1] == -1) {
                    if (n3 == -1) {
                        object2 = new DataInputStream(a.getClass().getResourceAsStream("/" + (String)object));
                    }
                    for (n11 = 0; n11 < a.length; ++n11) {
                        Object object5 = a[n11].a((DataInputStream)(n2 != 0 ? object2 : null), n2, a[n12][0], n12);
                        object4 = object5;
                        if (object5 == null) continue;
                        p.a[n12] = object4;
                        n6 = 1;
                        break;
                    }
                } else if (n2 != 0) {
                    for (n11 = 0; n11 < a.length; ++n11) {
                        boolean bl = a[n11].a((DataInputStream)object2, (String)(n2 == -1 ? object : null), n2, a[n12][0], n12, object4[1]);
                        n6 = bl ? 1 : 0;
                        if (!bl) {
                            continue;
                        }
                        break;
                    }
                }
                if (n6 == 0 && n3 != -1) {
                    p.a((DataInputStream)object2, n2);
                }
                if (object2 == null) continue;
                object3 = object;
                n5 = n3 + n2;
            }
            if (object2 != null) {
                ((FilterInputStream)object2).close();
            }
        }
        catch (IOException iOException) {}
        for (int i3 = 0; i3 < a.size(); ++i3) {
            int n13 = (Integer)a.elementAt(i3);
            p.a[n13] = true;
        }
        a.removeAllElements();
    }

    private static void g() {
        for (int i2 = 0; i2 < b.size(); ++i2) {
            int n2 = (Integer)b.elementAt(i2);
            if (!a[n2]) continue;
            short s2 = a[n2][0];
            for (int i3 = 0; i3 < a.length && !a[i3].a(s2, n2); ++i3) {
            }
            p.a[n2] = null;
            p.a[n2] = false;
        }
        b.removeAllElements();
    }

    private static short a(byte[] byArray, int n2) {
        return (short)(byArray[n2] << 8 | byArray[n2 + 1] & 0xFF);
    }

    private static DataInputStream a(int n2) {
        if (e[(n2 << 1) + 1] != 0) {
            InputStream inputStream = a.getClass().getResourceAsStream("/" + c[n2]);
            inputStream = new DataInputStream(inputStream);
            if (e[n2 << 1] != -1) {
                p.a((DataInputStream)inputStream, e[n2 << 1]);
            }
            return inputStream;
        }
        return null;
    }

    public final boolean a(DataInputStream dataInputStream, int n2) {
        switch (n2) {
            case -1: {
                int n3;
                a = new short[428];
                a = new byte[2282];
                a = new Image[46];
                for (n3 = 0; n3 < a.length; ++n3) {
                    p.a[n3] = -1;
                }
                for (n3 = 0; n3 < a.length; ++n3) {
                    p.a[n3] = -1;
                }
                break;
            }
            case 4: {
                o = dataInputStream.readByte();
                d = new String[dataInputStream.readShort()];
                break;
            }
            case 6: {
                short s2 = dataInputStream.readShort();
                b = new short[s2][];
                for (n2 = 0; n2 < s2; ++n2) {
                    int n4 = dataInputStream.readShort();
                    p.b[n2] = new short[n4];
                    for (int i2 = 0; i2 < n4; ++i2) {
                        p.b[n2][i2] = dataInputStream.readShort();
                    }
                }
                return true;
            }
            case 7: {
                return true;
            }
        }
        return false;
    }

    public final Object a(DataInputStream dataInputStream, int n2, int n3, int n4) {
        switch (n3) {
            case 2: {
                int n5 = dataInputStream.readByte();
                n2 = dataInputStream.readShort();
                n3 = dataInputStream.readShort();
                short s2 = dataInputStream.readShort();
                short s3 = dataInputStream.readShort();
                for (int i2 = 0; i2 < n3; ++i2) {
                    short s4 = dataInputStream.readShort();
                    p.a[s4 - 326 << 1] = (short)n4;
                    p.a[(s4 - 326 << 1) + 1] = (short)(dataInputStream.readShort() + 4);
                }
                ByteArrayOutputStream byteArrayOutputStream = new ByteArrayOutputStream(s2 + 4);
                DataOutputStream dataOutputStream = new DataOutputStream(byteArrayOutputStream);
                dataOutputStream.writeShort(n2);
                dataOutputStream.writeShort(n2 + n5);
                for (n5 = 0; n5 < s2; ++n5) {
                    dataOutputStream.writeByte(dataInputStream.readByte());
                }
                n5 = 0;
                for (n2 = 0; n2 < s3 << 3; ++n2) {
                    if ((n2 & 7) == 0) {
                        n5 = 7 * dataInputStream.readShort();
                        continue;
                    }
                    p.a[n5 + (n2 & 7) - 1] = dataInputStream.readByte();
                }
                return byteArrayOutputStream.toByteArray();
            }
            case 4: {
                int n6;
                n2 = dataInputStream.readShort();
                short s5 = dataInputStream.readShort();
                n3 = -999999;
                n4 = -999999;
                int n7 = 0;
                for (n6 = 0; n6 < o + 1; ++n6) {
                    if (n6 == 0) {
                        n3 = dataInputStream.readInt();
                        continue;
                    }
                    n7 = dataInputStream.readInt();
                    if (n6 != 1) continue;
                    n4 = n7 - n3;
                }
                p.a(dataInputStream, n3);
                for (n6 = 0; n6 < n2; ++n6) {
                    if (d[s5 + n6] == null) {
                        p.d[s5 + n6] = dataInputStream.readUTF();
                        continue;
                    }
                    dataInputStream.readUTF();
                }
                p.a(dataInputStream, n7 - (n3 + n4));
                return new short[]{s5, (short)n2};
            }
            case 3: {
                return new Object();
            }
        }
        return p.a((InputStream)dataInputStream, n2);
    }

    public final boolean a(DataInputStream object, String string, int n2, int n3, int n4, int n5) {
        switch (n3) {
            case 2: {
                int n6 = p.a((byte[])p.a(n4), 0) + n5;
                if (object != null) {
                    object = p.a((InputStream)object, n2);
                    p.a[n6] = Image.createImage((byte[])object, (int)0, (int)((Object)object).length);
                } else {
                    p.a[n6] = Image.createImage((String)("/" + string));
                }
                return true;
            }
            case 3: 
            case 8: {
                if (object != null) {
                    p.a(p.a((InputStream)object, n2), n4, null);
                } else {
                    p.a(null, n4, "/" + string);
                }
                return true;
            }
        }
        return false;
    }

    public final boolean a(int n2, int n3) {
        switch (n2) {
            case 2: {
                Object object = (byte[])a[n3];
                for (n2 = 0; n2 < a.length; n2 += 2) {
                    if (a[n2] != n3) continue;
                    p.a[n2] = -1;
                    p.a[n2 + 1] = -1;
                }
                n2 = p.a((byte[])object, 0);
                n3 = p.a((byte[])object, 2);
                for (int i2 = 0; i2 < a.length; i2 += 7) {
                    int n4 = a[i2 + 6];
                    if (n4 < n2 || n4 >= n3) continue;
                    for (n4 = 0; n4 < 7; ++n4) {
                        p.a[i2 + n4] = -1;
                    }
                }
                n2 = p.a((byte[])object, 2);
                for (n3 = (int)p.a((byte[])object, 0); n3 < n2; ++n3) {
                    p.a[n3] = null;
                }
                return true;
            }
            case 4: {
                Object object = (short[])a[n3];
                for (n2 = (v42721)[0]; n2 < object[1] + object[0]; ++n2) {
                    p.d[n2] = null;
                }
                return true;
            }
            case 3: {
                return true;
            }
        }
        return false;
    }

    public static void g(int n2) {
        short[] sArray = b[0];
        for (int i2 = 0; i2 < sArray.length; ++i2) {
            p.e(sArray[i2]);
        }
    }

    public static Image a(int n2) {
        if ((n2 = p.d(n2)) >= 0) {
            return a[n2];
        }
        return null;
    }

    public static void a(int n2, Image image) {
        if ((n2 = p.d(n2)) >= 0) {
            p.a[n2] = image;
        }
    }

    private static int d(int n2) {
        int n3;
        int n4;
        int n5;
        int n6;
        if (n2 == -1) {
            return -1;
        }
        if (n2 < -1) {
            return -1;
        }
        byte[] byArray = null;
        if (n2 < 326) {
            n6 = -99;
            n5 = 0;
            n4 = 0;
            if (a[n2 * 7 + 6] == -1) {
                return -1;
            }
        } else {
            n3 = (n2 - 326 << 1) + 1;
            if (n3 >= a.length) {
                return -1;
            }
            n4 = a[n3];
            n6 = a[n2 - 326 << 1];
            if (n6 < 0) {
                return -1;
            }
            byArray = (byte[])a[n6];
            if (n4 == -1 || byArray == null) {
                return -1;
            }
            n5 = byArray[n4++];
            n6 = n5 & 3;
            n5 = (n5 & 4) != 0 ? 2 : 1;
        }
        n3 = -1;
        if (n6 == 0 || n6 == -99) {
            if (n6 == 0) {
                n4 = n5 == 2 ? (n4 += 12) : (n4 += 6);
                n3 = (short)(byArray[n4] << 8 | byArray[n4 + 1] & 0xFF);
            } else {
                n3 = a[(n2 *= 7) + 6];
            }
        }
        return n3;
    }

    /*
     * Enabled aggressive block sorting
     */
    public static void a(int n2, int n3, int n4) {
        short s2;
        int n5;
        int n6;
        int n7;
        byte[] byArray = null;
        if (n4 < 326) {
            n7 = -99;
            n6 = 0;
            n5 = 0;
        } else {
            n5 = a[(n4 - 326 << 1) + 1];
            byArray = (byte[])a[a[n4 - 326 << 1]];
            s2 = byArray[n5++];
            n7 = s2 & 3;
            int n8 = n6 = (s2 & 4) != 0 ? 2 : 1;
        }
        if (n7 == 0 || n7 == -99) {
            Image image;
            if (n7 == 0) {
                if (n6 == 2) {
                    for (n4 = 0; n4 < 6; n5 += 2, ++n4) {
                        p.b[n4] = (short)(byArray[n5] << 8 | byArray[n5 + 1] & 0xFF);
                    }
                } else {
                    for (n4 = 0; n4 < 6; ++n4) {
                        p.b[n4] = byArray[n5++];
                    }
                }
                image = a[(short)(byArray[n5] << 8 | byArray[n5 + 1] & 0xFF)];
            } else {
                p.b[0] = (short)(a[n4 *= 7] & 0xFF);
                p.b[1] = (short)(a[n4 + 1] & 0xFF);
                p.b[2] = a[n4 + 2];
                p.b[3] = a[n4 + 3];
                p.b[4] = (short)(a[n4 + 4] & 0xFF);
                p.b[5] = (short)(a[n4 + 5] & 0xFF);
                image = a[a[n4 + 6]];
            }
            n2 -= b[2];
            n3 -= b[3];
            b.getClipX();
            b.getClipY();
            b.getClipWidth();
            b.getClipHeight();
            n4 = b[0];
            n7 = b[1];
            if (n4 <= 0) return;
            if (n7 <= 0) return;
            if (n != -1) {
                b.drawRegion(image, (int)b[4], (int)b[5], n4, n7, n, n2, n3, 20);
                n = -1;
                return;
            }
            b.drawRegion(image, (int)b[4], (int)b[5], n4, n7, 0, n2, n3, 20);
            return;
        }
        if (n7 != 1) return;
        s2 = (short)(byArray[n5 += n6 << 2] << 8 | byArray[n5 + 1] & 0xFF);
        n5 += 2;
        if (n6 == 2) {
            n4 = 0;
            while (n4 < s2) {
                p.a(n2 + (short)(byArray[n5] << 8 | byArray[n5 + 1] & 0xFF), n3 + (short)(byArray[n5 + 2] << 8 | byArray[n5 + 3] & 0xFF), (int)((short)(byArray[n5 + 4] << 8 | byArray[n5 + 5] & 0xFF)));
                n5 += 6;
                ++n4;
            }
            return;
        }
        n4 = 0;
        while (n4 < s2) {
            p.a(n2 + byArray[n5++], n3 + byArray[n5++], (int)byArray[n5++]);
            ++n4;
        }
    }

    public static void a(int n2, int n3, int n4, int n5) {
        n2 += p.a(n4, 2);
        n3 += p.a(n4, 3);
        if ((n5 & 1) == 1) {
            n2 -= p.a(n4, 0) / 2;
        }
        if ((n5 & 8) == 8) {
            n2 -= p.a(n4, 0);
        }
        if ((n5 & 2) == 2) {
            n3 -= p.a(n4, 1) / 2;
        }
        if ((n5 & 0x20) == 32) {
            n3 -= p.a(n4, 1);
        }
        p.a(n2, n3, n4);
    }

    public static void a(int n2, int n3, int n4, int n5, int n6) {
        n = 6;
        p.a(n2, n3, 12, 24);
    }

    public static int a(int n2, int n3) {
        int n4;
        if (n2 < 326) {
            if (n3 == 2 || n3 == 3) {
                return a[n2 * 7 + n3];
            }
            return a[n2 * 7 + n3] & 0xFF;
        }
        int n5 = a[(n2 - 326 << 1) + 1];
        byte[] byArray = (byte[])a[a[n2 - 326 << 1]];
        if ((n4 = ((n4 = byArray[n5++]) & 4) != 0 ? 2 : 1) == 2) {
            return (short)(byArray[n5 += n3 << 1] << 8 | byArray[n5 + 1] & 0xFF);
        }
        return byArray[n5 + n3];
    }

    public static int b(int n2, int n3) {
        int n4 = a[(n2 - 326 << 1) + 1];
        byte[] byArray = (byte[])a[a[n2 - 326 << 1]];
        int n5 = byArray[n4++];
        int n6 = n5 & 3;
        int n7 = n5 = (n5 & 4) != 0 ? 2 : 1;
        if (n6 == 1) {
            n6 = p.a(byArray, n4 += n5 << 2);
            n4 += 2;
            n4 += n5 * 3 * n6;
            n4 += (n5 << 1) * n3;
            if (n5 == 1) {
                return byArray[n4] << 16 & 0xFFFF0000 | byArray[n4 + 1] & 0xFFFF;
            }
            return p.a(byArray, n4) << 16 & 0xFFFF0000 | p.a(byArray, n4 + 2) & 0xFFFF;
        }
        return 0;
    }

    public static void b(int n2, int n3, int n4, int n5) {
        p.a(n2, n3, p.c(n4, n5));
    }

    public static int c(int n2, int n3) {
        int n4 = a[(n2 - 326 << 1) + 1];
        byte[] byArray = (byte[])a[a[n2 - 326 << 1]];
        n4 += 3;
        return byArray[n4 += n3 * 2] << 8 | byArray[n4 + 1] & 0xFF;
    }

    public static int b(int n2) {
        int n3 = a[(n2 - 326 << 1) + 1];
        byte[] byArray = (byte[])a[a[n2 - 326 << 1]];
        return p.a(byArray, ++n3);
    }

    public static void a(boolean bl) {
        j = true;
    }

    public static boolean c() {
        return j;
    }

    private static void a(byte[] byArray, int n2, String object) {
        if (byArray == null) {
            object = a.getClass().getResourceAsStream((String)object);
            try {
                byArray = p.a((InputStream)object, -1);
            }
            catch (Exception exception) {}
        }
        byte[] byArray2 = byArray;
        int n3 = n2;
        if (n3 >= 0) {
            p.a[n3] = byArray2;
            p.a[n3] = byArray2 != null;
        }
    }

    /*
     * WARNING - Removed try catching itself - possible behaviour change.
     */
    public static void c(int n2, int n3) {
        Object object = b;
        synchronized (object) {
            if (f) {
                g = n3 == -1 ? n2 : -1;
                return;
            }
            g = -1;
            byte[] byArray = (byte[])p.a(n2);
            if (byArray == null) {
                return;
            }
            p.d();
            if (j) {
                try {
                    if (n3 == -1) {
                        n3 = -1;
                        r = n2;
                    }
                    a = Manager.createPlayer((InputStream)new ByteArrayInputStream(byArray), (String)"audio/midi");
                    a.prefetch();
                    VolumeControl volumeControl = (VolumeControl)a.getControl("VolumeControl");
                    if (volumeControl != null) {
                        volumeControl.setLevel(q);
                    }
                    a.setLoopCount(n3);
                    a.start();
                }
                catch (Exception exception) {}
            }
            return;
        }
    }

    /*
     * WARNING - Removed try catching itself - possible behaviour change.
     */
    public static void d() {
        Object object = b;
        synchronized (object) {
            g = -1;
            if (j && a != null) {
                try {
                    if (a.getState() != 0) {
                        a.stop();
                        a.deallocate();
                        a.close();
                    }
                }
                catch (Exception exception) {}
                r = -1;
                a = null;
            }
            return;
        }
    }

    /*
     * WARNING - Removed try catching itself - possible behaviour change.
     */
    public static void h(int n2) {
        Object object = b;
        synchronized (object) {
            t = 0;
            v = 0;
            u = 0;
            if (n2 == 1) {
                a = new p();
                new Thread(a).start();
                return;
            }
            if (a == null) {
                return;
            }
            if (n2 == 2 || n2 == 4) {
                if (!e) {
                    e = true;
                    f = true;
                    n2 = r;
                    p.d();
                    g = n2;
                }
            } else if ((n2 == 3 || n2 == 5) && e) {
                e = false;
                b.notifyAll();
            }
            return;
        }
    }

    /*
     * Exception decompiling
     */
    public final void run() {
        /*
         * This method has failed to decompile.  When submitting a bug report, please provide this stack trace, and (if you hold appropriate legal rights) the relevant class file.
         * 
         * org.benf.cfr.reader.util.ConfusedCFRException: Started 2 blocks at once
         *     at org.benf.cfr.reader.bytecode.analysis.opgraph.Op04StructuredStatement.getStartingBlocks(Op04StructuredStatement.java:412)
         *     at org.benf.cfr.reader.bytecode.analysis.opgraph.Op04StructuredStatement.buildNestedBlocks(Op04StructuredStatement.java:487)
         *     at org.benf.cfr.reader.bytecode.analysis.opgraph.Op03SimpleStatement.createInitialStructuredBlock(Op03SimpleStatement.java:736)
         *     at org.benf.cfr.reader.bytecode.CodeAnalyser.getAnalysisInner(CodeAnalyser.java:850)
         *     at org.benf.cfr.reader.bytecode.CodeAnalyser.getAnalysisOrWrapFail(CodeAnalyser.java:278)
         *     at org.benf.cfr.reader.bytecode.CodeAnalyser.getAnalysis(CodeAnalyser.java:201)
         *     at org.benf.cfr.reader.entities.attributes.AttributeCode.analyse(AttributeCode.java:94)
         *     at org.benf.cfr.reader.entities.Method.analyse(Method.java:531)
         *     at org.benf.cfr.reader.entities.ClassFile.analyseMid(ClassFile.java:1055)
         *     at org.benf.cfr.reader.entities.ClassFile.analyseTop(ClassFile.java:942)
         *     at org.benf.cfr.reader.Driver.doJarVersionTypes(Driver.java:257)
         *     at org.benf.cfr.reader.Driver.doJar(Driver.java:139)
         *     at org.benf.cfr.reader.CfrDriverImpl.analyse(CfrDriverImpl.java:76)
         *     at org.benf.cfr.reader.Main.main(Main.java:54)
         */
        throw new IllegalStateException("Decompilation failed");
    }

    private static void h() {
        b = 0L;
        a = 0;
        p.c();
    }

    private static boolean d() {
        return a.size() > 0 || b.size() > 0 || f > 0;
    }

    /*
     * WARNING - Removed try catching itself - possible behaviour change.
     */
    private void i() {
        g = true;
        new Thread(a).start();
        p.e();
        int n2 = 0;
        int n3 = 0;
        while (n2 < f || !a.isEmpty() || !b.isEmpty()) {
            if (!b.isEmpty()) {
                p.g();
                System.gc();
            }
            if (!a.isEmpty()) {
                p.f();
                System.gc();
            }
            if (n2 >= f || (n3 = a.b(b[n2], n3)) != 0) continue;
            ++n2;
        }
        Object object = a;
        synchronized (object) {
            g = false;
            a.notify();
        }
        f = 0;
        p.h();
    }

    private static void j() {
        int n2 = a.getWidth();
        int n3 = a.getHeight();
        if (b != n2 || c != n3) {
            b = n2;
            c = n3;
            if (a != null) {
                p.i(2);
            }
        }
    }

    private void j(int n2) {
        j = n2;
        Graphics graphics = a.getGraphics();
        this.b(graphics);
        a.flushGraphics();
        j = 0;
    }

    private static void k() {
        long l2 = System.currentTimeMillis();
        if (b != 0L) {
            a = (int)(l2 - b);
            if ((a /= i) > h) {
                a = h;
            }
        } else {
            a = 0;
        }
        b = l2;
        l = false;
        for (int i2 = 0; i2 < i; ++i2) {
            int n2;
            for (n2 = 0; n2 < w; ++n2) {
                a.b(f[n2]);
                p.f[n2] = -1;
            }
            w = 0;
            v = t | u;
            u = 0;
            n2 = a.a(0);
            while (n2 != 0) {
                n2 = a.a(n2);
            }
            if (p.d()) break;
        }
    }

    /*
     * WARNING - Removed try catching itself - possible behaviour change.
     */
    private static void l() {
        Object object = b;
        synchronized (object) {
            if (!p.d()) {
                f = false;
                p.i(3);
            }
            if (e) {
                try {
                    b.wait();
                }
                catch (InterruptedException interruptedException) {}
            }
        }
        p.h();
    }

    private void b(Graphics graphics) {
        if (j == 0) {
            return;
        }
        try {
            p p2 = graphics;
            a = p2;
            b = p2;
            p2.setClip(0, 0, b, c);
            if (a == null) {
                p2.setColor(0);
                p2.fillRect(0, 0, b, c);
            } else {
                if (!c) {
                    return;
                }
                int n2 = a.a(0, j);
                while (n2 != 0) {
                    n2 = a.a(n2, j);
                }
            }
            p2.setClip(0, 0, b, c);
            for (int i2 = 0; i2 < 3; ++i2) {
                String string = b[i2];
                if (string == null) continue;
                int n3 = p.c(i2);
                int n4 = 0;
                int n5 = 0;
                if ((n3 & 4) == 4) {
                    n4 = 2;
                }
                if ((n3 & 8) == 8) {
                    n4 = b - 2;
                }
                if ((n3 & 1) == 1) {
                    n4 = b >> 1;
                }
                if ((n3 & 0x10) == 16) {
                    n5 = 2;
                }
                if ((n3 & 0x20) == 32) {
                    n5 = c - 2;
                }
                if ((n3 & 2) == 2) {
                    n5 = c >> 1;
                }
                n.a(string, c[i2], n4, n5, n3);
            }
            return;
        }
        catch (Throwable throwable) {
            return;
        }
    }

    private static int e(int n2) {
        if (n2 == -7) {
            return 5;
        }
        if (n2 == -6) {
            return 6;
        }
        if (n2 == -5) {
            return 7;
        }
        if (n2 == -8) {
            return 23;
        }
        switch (n2) {
            case 48: {
                return 10;
            }
            case 49: {
                return 11;
            }
            case 50: {
                return 12;
            }
            case 51: {
                return 13;
            }
            case 52: {
                return 14;
            }
            case 53: {
                return 15;
            }
            case 54: {
                return 16;
            }
            case 55: {
                return 17;
            }
            case 56: {
                return 18;
            }
            case 57: {
                return 19;
            }
            case 42: {
                return 20;
            }
            case 35: {
                return 21;
            }
        }
        try {
            n2 = a.getGameAction(n2);
        }
        catch (IllegalArgumentException illegalArgumentException) {
            return -1;
        }
        switch (n2) {
            case 6: {
                return 2;
            }
            case 1: {
                return 1;
            }
            case 2: {
                return 3;
            }
            case 5: {
                return 4;
            }
        }
        return -1;
    }

    private static void d(int n2, int n3) {
        block26: {
            if (l) {
                return;
            }
            if (!c) {
                return;
            }
            try {
                block27: {
                    block30: {
                        block28: {
                            block29: {
                                int n4;
                                if ((n3 & 8) == 8) {
                                    n2 = p.e(n2);
                                }
                                if (s == 2) {
                                    if (n2 == 15) {
                                        n2 = 7;
                                    } else if (n2 == 12) {
                                        n2 = 1;
                                    } else if (n2 == 14) {
                                        n2 = 3;
                                    } else if (n2 == 18) {
                                        n2 = 2;
                                    } else if (n2 == 16) {
                                        n2 = 4;
                                    }
                                }
                                if (s != 3 || (n2 < 10 || n2 > 21) && n2 != 21) break block27;
                                if ((n3 & 1) != 1) break block28;
                                if (n2 == 21) break block29;
                                long l2 = System.currentTimeMillis();
                                y = l2 - c < 700L && n2 == x ? ++y : 0;
                                n3 = n2 - 10;
                                if (a[n3].length == 0 || n3 < 0) break block30;
                                c = l2;
                                x = n2;
                                n3 = n4 = a[n3][y % a[n3].length];
                                if (!(n2 == z && n3 == a || m)) {
                                    z = n2;
                                    a = (char)n3;
                                    d = System.currentTimeMillis();
                                    m = true;
                                }
                            }
                            return;
                        }
                        if ((n3 & 2) == 2) {
                            long l3;
                            if (n2 >= 10 && n2 <= 19 && (l3 = System.currentTimeMillis()) - c > 1200L) {
                                y = 0;
                                c = 0L;
                            }
                            p.k(n2);
                        }
                    }
                    return;
                }
                if ((n3 & 1) == 1) {
                    if (w < 20) {
                        p.f[p.w] = n2;
                        ++w;
                    }
                    if (n2 != -1) {
                        t |= 1 << n2;
                        u |= 1 << n2;
                        break block26;
                    }
                } else if ((n3 & 2) == 2 && n2 != -1) {
                    t &= ~(1 << n2);
                }
                return;
            }
            catch (Throwable throwable) {}
        }
    }

    private static void k(int n2) {
        if (n2 == z && m) {
            m = false;
            z = 999;
            a = (char)32;
        }
    }

    protected final void keyPressed(int n2) {
        p.d(n2, 9);
    }

    protected final void keyReleased(int n2) {
        p.d(n2, 10);
    }

    protected final void showNotify() {
        p.h(5);
    }

    protected final void hideNotify() {
        p.h(4);
    }

    public final void paint(Graphics graphics) {
        this.b(graphics);
    }

    public final void commandAction(Command command, Displayable displayable) {
    }

    static {
        e = false;
        f = false;
        g = false;
        h = false;
        g = -1;
        i = false;
        h = 500;
        i = 1;
        j = 0;
        a = new char[]{'\n', ' ', '-'};
        b = 240;
        c = 320;
        a = null;
        b = null;
        a = new Font[]{Font.getFont((int)64, (int)2, (int)0), Font.getFont((int)64, (int)1, (int)0), Font.getFont((int)64, (int)0, (int)0)};
        d = new int[2];
        k = 0;
        m = -1;
        b = new short[6];
        n = -1;
        p = 0;
        j = false;
        k = false;
        a = null;
        b = null;
        q = 60;
        r = -1;
        s = 1;
        t = 0;
        u = 0;
        v = 0;
        c = 0L;
        x = 0;
        y = 0;
        d = 0L;
        m = false;
        a = new char[][]{{' ', '0'}, {'.', ',', '?', '!', '\'', '-', '(', ')', '@', '/', ':', '_', '1'}, {'a', 'b', 'c', '\u00e4', '\u00e5', '2'}, {'d', 'e', 'f', '3'}, {'g', 'h', 'i', '4'}, {'j', 'k', 'l', '5'}, {'m', 'n', 'o', '\u00f6', '6'}, {'p', 'q', 'r', 's', '7'}, {'t', 'u', 'v', '8'}, {'w', 'x', 'y', 'z', '9'}, new char[0], new char[0]};
        b = null;
        n = false;
    }
}

