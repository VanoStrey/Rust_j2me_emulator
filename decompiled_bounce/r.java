/*
 * Decompiled with CFR 0.152.
 */
import java.io.DataInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.Serializable;
import java.util.Vector;

/*
 * Duplicate member names - consider using --renamedupmembers true
 */
public final class r {
    private static r a;
    private static DataInputStream a;
    private static String a;

    private r() {
    }

    private static String a(String string, String string2, String string3) {
        int n2;
        do {
            if ((n2 = string.indexOf(string2)) < 0) continue;
            string = string.substring(0, n2) + string3 + string.substring(n2 + string2.length());
        } while (n2 >= 0);
        return string;
    }

    public static String a(int n2) {
        return r.a(n2, null);
    }

    public static synchronized String a(int n2, String[] stringArray) {
        if (a == null) {
            a = System.getProperty("microedition.locale");
        }
        try {
            if (a == null) {
                a = new r();
            }
            if (a == null) {
                InputStream inputStream = a.getClass().getResourceAsStream("/lang." + a);
                if (inputStream == null) {
                    inputStream = a.getClass().getResourceAsStream("/lang.xx");
                }
                if (inputStream == null) {
                    return "X";
                }
                a = new DataInputStream(inputStream);
                a.mark(512);
            }
            a.skipBytes(n2 << 1);
            int n3 = a.readUnsignedShort();
            a.skipBytes(n3 - (n2 << 1) - 2);
            String string = a.readUTF();
            if (!a.markSupported()) {
                a.close();
                a = null;
            } else {
                try {
                    a.reset();
                }
                catch (IOException iOException) {
                    a.close();
                    a = null;
                }
            }
            if (stringArray != null) {
                if (stringArray.length == 1) {
                    string = r.a(string, "%U", stringArray[0]);
                } else {
                    for (n3 = 0; n3 < stringArray.length; ++n3) {
                        string = r.a(string, "%" + n3 + "U", stringArray[n3]);
                    }
                }
            }
            return string;
        }
        catch (IOException iOException) {
            a = null;
            return "E";
        }
    }

    private static boolean a(String string, String string2, int n2, int n3) {
        block4: while (true) {
            if (n2 == string.length() && n3 == string2.length()) {
                return true;
            }
            if (n2 == string.length() || n3 == string2.length()) {
                return false;
            }
            switch (string2.charAt(n3)) {
                case '?': {
                    ++n2;
                    ++n3;
                    continue block4;
                }
                case '*': {
                    if (n3 == string2.length() - 1) {
                        return true;
                    }
                    if (r.a(string, string2, n2, n3 + 1)) {
                        return true;
                    }
                    ++n2;
                    continue block4;
                }
            }
            if (string.charAt(n2) != string2.charAt(n3)) break;
            ++n2;
            ++n3;
        }
        return false;
    }

    private static StringBuffer a(InputStream inputStream) {
        StringBuffer stringBuffer = new StringBuffer();
        try {
            int n2 = inputStream.read();
            if ((char)n2 != ' ') {
                return stringBuffer;
            }
            while ((n2 = inputStream.read()) >= 0) {
                if ((char)n2 == '\r') continue;
                if ((char)n2 == '\n') {
                    stringBuffer.append((Object)r.a(inputStream));
                    break;
                }
                stringBuffer.append((char)n2);
            }
        }
        catch (IOException iOException) {}
        return stringBuffer;
    }

    static {
        block11: {
            InputStream inputStream;
            a = null;
            a = null;
            String string = System.getProperty("microedition.platform");
            boolean bl = false;
            Serializable serializable = new StringBuffer();
            if (a == null) {
                a = new r();
            }
            if ((inputStream = a.getClass().getResourceAsStream("/META-INF/MANIFEST.MF")) != null) {
                int n2;
                block2: while (true) {
                    try {
                        int n3;
                        while ((n3 = inputStream.read()) >= 0) {
                            if ((char)n3 == '\r') continue;
                            if ((char)n3 == '\n') {
                                if (((StringBuffer)serializable).toString().trim().startsWith("Nokia-Platform:")) {
                                    ((StringBuffer)serializable).append((Object)r.a(inputStream));
                                    break block2;
                                }
                                ((StringBuffer)serializable).delete(0, ((StringBuffer)serializable).length());
                                continue;
                            }
                            ((StringBuffer)serializable).append((char)n3);
                        }
                        break block11;
                    }
                    catch (IOException iOException) {
                        continue;
                    }
                    break;
                }
                String string2 = ((StringBuffer)serializable).toString().trim().substring(15);
                serializable = new Vector();
                while ((n2 = string2.indexOf("@")) != -1) {
                    ((Vector)serializable).addElement(string2.substring(0, n2));
                    string2 = string2.substring(n2 + 1, string2.length());
                }
                ((Vector)serializable).addElement(string2);
                for (n2 = 0; n2 < ((Vector)serializable).size(); ++n2) {
                    string2 = string;
                    String string3 = ((String)((Vector)serializable).elementAt(n2)).trim();
                    if (!r.a(string2, string3, 0, 0)) continue;
                    bl = true;
                    break;
                }
            }
            if (!bl) {
                System.exit(0);
            }
        }
        a = null;
    }
}

