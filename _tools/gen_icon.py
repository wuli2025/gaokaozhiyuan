#!/usr/bin/env python3
"""
把 BrandLogo.vue 的 SVG 学士帽图标重绘成各种尺寸 PNG + .ico
用于替换 src-tauri/icons/ 下的默认图标
"""
from PIL import Image, ImageDraw, ImageFilter
import math, os

SIZE = 1024
RX = 280  # 圆角半径 (~28%)

# 颜色
c_red_top    = (224,  72,  59)  # #e0483b
c_red_mid    = (210,  58,  44)  # #d23a2c
c_red_bot    = (168,  42,  35)  # #a82a23
c_white      = (255, 255, 255)
c_gold       = (255, 213, 107)  # #ffd56b

icons_dir = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")


def lerp_color(a, b, t):
    return tuple(int(a[i] + (b[i]-a[i])*t) for i in range(3))


def make_gradient(draw, width, height, c1, c2, c3, diagonal=True):
    """对角线渐变背景"""
    for y in range(height):
        for x in range(width):
            # 对角线参数 0~1
            t = (x + y) / (width + height)
            if t < 0.55:
                t2 = t / 0.55
                col = lerp_color(c1, c2, t2)
            else:
                t2 = (t - 0.55) / 0.45
                col = lerp_color(c2, c3, t2)
            draw.point((x, y), fill=col)


def draw_rounded_rect(img, draw, xy, radius, fill):
    """画圆角矩形"""
    x1, y1, x2, y2 = xy
    r = min(radius, (x2-x1)//2, (y2-y1)//2)
    # 主体矩形（去圆角）
    draw.rectangle([x1+r, y1, x2-r, y2], fill=fill)
    draw.rectangle([x1, y1+r, x2, y2-r], fill=fill)
    # 四个圆角
    draw.ellipse([x1, y1, x1+r*2, y1+r*2], fill=fill)
    draw.ellipse([x2-r*2, y1, x2, y1+r*2], fill=fill)
    draw.ellipse([x1, y2-r*2, x1+r*2, y2], fill=fill)
    draw.ellipse([x2-r*2, y2-r*2, x2, y2], fill=fill)


def draw_shine(draw, width, height, radius):
    """顶部光泽（白色半透明渐变）"""
    x1, y1 = 30, 30
    x2 = width - 30
    y2 = height // 2 + 50
    r = min(radius, (x2-x1)//2, (y2-y1)//2)
    # 简化为顶部半透明白色覆盖
    for y in range(y1, y2):
        alpha = int(72 * (1 - (y - y1) / (y2 - y1)))
        if alpha > 0:
            for x in range(x1, x2):
                if x < x1+r and y < y1+r:
                    if (x-x1-r)**2 + (y-y1-r)**2 > r*r:
                        continue
                if x > x2-r and y < y1+r:
                    if (x-x2+r)**2 + (y-y1-r)**2 > r*r:
                        continue
                draw.point((x, y), fill=(255, 255, 255, alpha))


def draw_cap(img, draw):
    """画学士帽 (基于 SVG 路径缩放)"""
    scale = SIZE / 100.0
    def s(v):
        return v * scale

    # 帽顶板（白色多边形）
    top_points = [
        (s(50),  s(27)),
        (s(86),  s(44)),
        (s(50),  s(61)),
        (s(14),  s(44)),
    ]
    draw.polygon(top_points, fill=c_white)

    # 帽身（白色路径）
    # M28 52 L28 66 Q50 79 72 66 L72 52 L50 62 Z
    x1, y1 = s(28), s(52)
    x2, y2 = s(28), s(66)
    x3, y3 = s(50), s(79)
    x4, y4 = s(72), s(66)
    x5, y5 = s(72), s(52)
    x6, y6 = s(50), s(62)

    # 简化为填充多边形（帽身形状）
    body_points = [
        (x1, y1), (x2, y2),
        # 曲线部分用多点近似
        (s(35), s(76)), (s(42), s(78)), (s(50), s(79)),
        (s(58), s(78)), (s(65), s(76)),
        (x4, y4), (x5, y5), (x6, y6),
    ]
    draw.polygon(body_points, fill=c_white)

    # 流苏线条
    sx, sy = s(86), s(44)
    ex, ey = s(86), s(67)
    draw.line([(sx, sy), (ex, ey)], fill=c_gold, width=max(3, int(s(3.4))))

    # 流苏圆
    cx, cy = s(86), s(71)
    r = max(4, int(s(4.4)))
    draw.ellipse([cx-r, cy-r, cx+r, cy+r], fill=c_gold)


def generate():
    # 1. 主图 1024x1024
    img = Image.new('RGBA', (SIZE, SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # 背景渐变
    make_gradient(draw, SIZE, SIZE, c_red_top, c_red_mid, c_red_bot)

    # 转为 RGBA 以便叠加 alpha
    bg = img.copy()
    img = Image.new('RGBA', (SIZE, SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # 圆角矩形裁剪蒙版
    mask = Image.new('L', (SIZE, SIZE), 0)
    mask_draw = ImageDraw.Draw(mask)
    draw_rounded_rect(mask, mask_draw, (0, 0, SIZE, SIZE), RX, 255)

    # 应用蒙版到渐变背景
    bg.putalpha(mask)

    # 光泽
    shine = Image.new('RGBA', (SIZE, SIZE), (0, 0, 0, 0))
    shine_draw = ImageDraw.Draw(shine)
    draw_shine(shine_draw, SIZE, SIZE, RX)
    # 只对顶部区域裁圆角
    shine_mask = Image.new('L', (SIZE, SIZE), 0)
    smd = ImageDraw.Draw(shine_mask)
    smd.rectangle([0, 0, SIZE, SIZE//2+50], fill=255)
    shine.putalpha(ImageChops.multiply(shine_mask, mask))

    # 合并背景 + 光泽
    result = Image.alpha_composite(bg, shine)

    # 学士帽
    cap = Image.new('RGBA', (SIZE, SIZE), (0, 0, 0, 0))
    cap_draw = ImageDraw.Draw(cap)
    draw_cap(cap, cap_draw)
    result = Image.alpha_composite(result, cap)

    return result


def resize_to(img, size):
    return img.resize((size, size), Image.LANCZOS)


def save_ico(img, path):
    """生成包含多尺寸的 .ico"""
    sizes = [256, 128, 64, 48, 32, 16]
    imgs = []
    for s in sizes:
        if s <= img.width:
            imgs.append(resize_to(img, s).convert('RGBA'))
    # 用第一个（最大）作为主图，其余附加
    # Pillow 的 save 支持 .ico 多帧
    imgs[0].save(path, format='ICO', sizes=[(i.width, i.height) for i in imgs])


if __name__ == '__main__':
    from PIL import ImageChops

    print("生成 1024x1024 图标...")
    main = generate()

    # 保存源图
    os.makedirs(icons_dir, exist_ok=True)
    main.save(os.path.join(icons_dir, 'icon.png'), 'PNG')
    print(f"  → icons/icon.png")

    # 保存 .ico (Windows 安装包用)
    save_ico(main, os.path.join(icons_dir, 'icon.ico'))
    print(f"  → icons/icon.ico")

    # 其他尺寸 PNG
    for sz in [32, 128, 256, 512]:
        r = resize_to(main, sz)
        r.save(os.path.join(icons_dir, f'{sz}x{sz}.png'), 'PNG')
        print(f"  → icons/{sz}x{sz}.png")

    # StoreLogo (Windows 商店风格)
    r = resize_to(main, 256)
    r.save(os.path.join(icons_dir, 'StoreLogo.png'), 'PNG')
    print(f"  → icons/StoreLogo.png")

    print("✅ 图标生成完成")
