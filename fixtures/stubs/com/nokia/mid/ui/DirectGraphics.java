package com.nokia.mid.ui;

import javax.microedition.lcdui.Image;

public interface DirectGraphics {
    public static final int TYPE_INT_8888_ARGB = 1;
    public static final int TYPE_USHORT_4444_ARGB = 2;
    public static final int TYPE_USHORT_565_RGB = 3;
    public static final int TYPE_USHORT_444_RGB = 4;
    public static final int TYPE_USHORT_1555_ARGB = 5;
    public static final int TYPE_INT_888_RGB = 6;

    public static final int FLIP_HORIZONTAL = 0x2000;
    public static final int FLIP_VERTICAL = 0x4000;
    public static final int ROTATE_90 = 90;
    public static final int ROTATE_180 = 180;
    public static final int ROTATE_270 = 270;

    public void setARGBColor(int argbColor);
    public void drawPixels(int[] pixels, boolean transparency, int offset, int scanlength, int x, int y, int width, int height, int manipulation, int format);
    public void drawPixels(short[] pixels, boolean transparency, int offset, int scanlength, int x, int y, int width, int height, int manipulation, int format);
    public void getPixels(int[] pixels, int offset, int scanlength, int x, int y, int width, int height, int format);
    public void getPixels(short[] pixels, int offset, int scanlength, int x, int y, int width, int height, int format);
    public void drawImage(Image img, int x, int y, int anchor, int manipulation);
    public void drawPolygon(int[] xPoints, int xOffset, int[] yPoints, int yOffset, int nPoints, int argbColor);
    public void fillPolygon(int[] xPoints, int xOffset, int[] yPoints, int yOffset, int nPoints, int argbColor);
}
