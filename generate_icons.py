#!/usr/bin/env python3
import os
import math
from PIL import Image, ImageDraw

OUTPUT_DIRS = [
    "/home/karl/Proyectos/Omnicraft-v2/public/minecraft-icons",
    "/home/karl/Games/Omnicraft-v2/icons"
]

def render_grid_to_image(pixel_map, size=64):
    """pixel_map is a list of lists of (R,G,B,A)"""
    h = len(pixel_map)
    w = len(pixel_map[0])
    cell_w = size / w
    cell_h = size / h
    
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    for y in range(h):
        for x in range(len(pixel_map[y])):
            c = pixel_map[y][x]
            if c and c[3] > 0:
                x0 = int(round(x * cell_w))
                y0 = int(round(y * cell_h))
                x1 = int(round((x + 1) * cell_w))
                y1 = int(round((y + 1) * cell_h))
                draw.rectangle([x0, y0, x1 - 1, y1 - 1], fill=c)
    return img

def gen_steve():
    SKIN_LIGHT = (219, 162, 130, 255)
    SKIN_DARK = (184, 130, 100, 255)
    HAIR = (74, 46, 28, 255)
    EYE_WHITE = (245, 245, 255, 255)
    EYE_PUPIL = (45, 62, 142, 255)
    NOSE = (162, 107, 83, 255)
    MOUTH = (103, 56, 36, 255)
    SHADOW = (130, 80, 50, 255)
    
    H, L, D, W, P, N, M, S = HAIR, SKIN_LIGHT, SKIN_DARK, EYE_WHITE, EYE_PUPIL, NOSE, MOUTH, SHADOW
    grid = [
        [H, H, H, H, H, H, H, H],
        [H, H, H, H, H, H, H, H],
        [H, H, L, L, L, L, H, H],
        [L, L, L, L, L, L, L, L],
        [L, W, P, L, L, P, W, L],
        [L, L, L, N, N, L, L, L],
        [L, L, M, M, M, M, L, L],
        [L, L, S, S, S, S, L, L],
    ]
    return render_grid_to_image(grid, 64)

def gen_alex():
    HAIR = (180, 95, 45, 255)
    HAIR_LIGHT = (210, 115, 55, 255)
    SKIN = (235, 185, 155, 255)
    SKIN_DARK = (205, 150, 120, 255)
    EYE_WHITE = (245, 245, 255, 255)
    EYE_GREEN = (45, 130, 75, 255)
    LIPS = (185, 115, 95, 255)
    
    H, HL, S, SD, W, G, LP = HAIR, HAIR_LIGHT, SKIN, SKIN_DARK, EYE_WHITE, EYE_GREEN, LIPS
    grid = [
        [H, H, HL, H, H, HL, H, H],
        [H, HL, H, H, H, H, HL, H],
        [H, S, S, S, S, S, S, H],
        [S, S, S, S, S, S, S, S],
        [S, W, G, S, S, G, W, S],
        [S, S, S, SD, SD, S, S, S],
        [H, S, LP, LP, LP, LP, S, H],
        [H, S, S, S, S, S, S, H],
    ]
    return render_grid_to_image(grid, 64)

def gen_creeper():
    G1 = (70, 185, 50, 255)
    G2 = (50, 150, 35, 255)
    G3 = (35, 120, 25, 255)
    G4 = (90, 210, 65, 255)
    B  = (18, 30, 18, 255)
    
    grid = [
        [G1, G2, G4, G1, G2, G1, G4, G2],
        [G2, G4, G1, G2, G4, G2, G1, G3],
        [G1, B,  B,  G1, G2, B,  B,  G1],
        [G4, B,  B,  G3, G1, B,  B,  G2],
        [G2, G1, G2, B,  B,  G2, G4, G1],
        [G1, G3, B,  B,  B,  B,  G1, G2],
        [G3, G2, B,  B,  B,  B,  G2, G4],
        [G1, G4, B,  G2, G1, B,  G4, G1],
    ]
    return render_grid_to_image(grid, 64)

def gen_skeleton():
    W1 = (220, 220, 220, 255)
    W2 = (180, 180, 180, 255)
    W3 = (145, 145, 145, 255)
    B  = (25, 25, 28, 255)
    
    grid = [
        [W1, W2, W1, W2, W2, W1, W2, W1],
        [W2, W1, W2, W1, W1, W2, W1, W2],
        [W1, B,  B,  W2, W1, B,  B,  W1],
        [W2, B,  B,  W3, W2, B,  B,  W2],
        [W1, W2, W1, B,  B,  W1, W2, W1],
        [W2, W3, B,  B,  B,  B,  W3, W2],
        [W1, W2, B,  W1, W1, B,  W2, W1],
        [W2, W1, W2, W2, W1, W2, W1, W2],
    ]
    return render_grid_to_image(grid, 64)

def gen_zombie():
    Z1 = (60, 115, 45, 255)
    Z2 = (45, 90, 35, 255)
    Z3 = (35, 75, 28, 255)
    H  = (25, 45, 25, 255)
    B  = (15, 20, 15, 255)
    
    grid = [
        [H,  H,  H,  H,  H,  H,  H,  H],
        [H,  H,  H,  H,  H,  H,  H,  H],
        [H,  Z1, Z2, Z1, Z2, Z1, Z2, H],
        [Z1, Z2, Z1, Z3, Z2, Z1, Z2, Z1],
        [Z2, B,  B,  Z1, Z2, B,  B,  Z2],
        [Z1, Z3, Z2, Z1, Z3, Z2, Z1, Z1],
        [Z2, Z1, B,  B,  B,  B,  Z1, Z2],
        [Z3, Z2, Z1, Z2, Z3, Z1, Z2, Z3],
    ]
    return render_grid_to_image(grid, 64)

def gen_enderman():
    B1 = (22, 22, 26, 255)
    B2 = (14, 14, 18, 255)
    B3 = (32, 32, 38, 255)
    P1 = (210, 80, 240, 255)
    P2 = (140, 30, 180, 255)
    W  = (255, 220, 255, 255)
    
    grid = [
        [B1, B2, B1, B3, B2, B1, B2, B1],
        [B2, B1, B3, B2, B1, B3, B1, B2],
        [B1, B3, B2, B1, B2, B1, B3, B1],
        [B2, B1, B1, B2, B3, B2, B1, B2],
        [P2, P1, W,  B1, B2, W,  P1, P2],
        [B3, B2, B1, B2, B1, B3, B2, B1],
        [B1, B2, B3, B1, B2, B1, B2, B3],
        [B2, B1, B2, B3, B1, B2, B1, B2],
    ]
    return render_grid_to_image(grid, 64)

def gen_isometric_block(theme="grass"):
    size = 64
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    cx, cy = 32, 32
    w = 26
    h = 15
    dh = 26
    
    # Vertices
    p_top = (cx, cy - h - 4)
    p_right = (cx + w, cy - 4)
    p_left = (cx - w, cy - 4)
    p_center = (cx, cy + h - 4)
    
    p_b_left = (cx - w, cy - 4 + dh)
    p_b_center = (cx, cy + h - 4 + dh)
    p_b_right = (cx + w, cy - 4 + dh)
    
    if theme == "grass":
        c_top = (95, 175, 55, 255)
        c_left = (120, 85, 50, 255)
        c_right = (95, 65, 40, 255)
        border = (40, 30, 20, 255)
        
        draw.polygon([p_top, p_right, p_center, p_left], fill=c_top, outline=border)
        draw.polygon([p_left, p_center, p_b_center, p_b_left], fill=c_left, outline=border)
        draw.polygon([p_center, p_right, p_b_right, p_b_center], fill=c_right, outline=border)
        
        # Grass overhang on sides
        draw.polygon([(cx - w, cy - 4), (cx - 16, cy + 2), (cx - 8, cy + 7), (cx, cy + h - 4), (cx, cy + h - 8), (cx - 16, cy - 2), (cx - w, cy - 4)], fill=(80, 150, 45, 255))
        draw.polygon([(cx, cy + h - 4), (cx + 10, cy + 8), (cx + 20, cy + 1), (cx + w, cy - 4), (cx + w, cy - 7), (cx + 10, cy + 4), (cx, cy + h - 4)], fill=(70, 130, 40, 255))

    elif theme == "crafting_table":
        c_top = (185, 135, 80, 255)
        c_left = (145, 100, 60, 255)
        c_right = (120, 80, 45, 255)
        border = (50, 35, 20, 255)
        
        draw.polygon([p_top, p_right, p_center, p_left], fill=c_top, outline=border)
        draw.polygon([p_left, p_center, p_b_center, p_b_left], fill=c_left, outline=border)
        draw.polygon([p_center, p_right, p_b_right, p_b_center], fill=c_right, outline=border)
        
        # 3x3 crafting grid simulation on top
        draw.line([(cx - 13, cy - 2), (cx + 13, cy - 6)], fill=(110, 75, 40, 255), width=2)
        draw.line([(cx - 13, cy - 6), (cx + 13, cy - 2)], fill=(110, 75, 40, 255), width=2)
        # Tools detail
        draw.rectangle([cx - 20, cy + 6, cx - 8, cy + 22], outline=(70, 50, 30, 255), fill=(110, 75, 45, 255))
        draw.rectangle([cx + 8, cy + 6, cx + 20, cy + 22], outline=(60, 40, 25, 255), fill=(95, 65, 35, 255))

    elif theme == "diamond_block":
        c_top = (110, 240, 240, 255)
        c_left = (70, 190, 200, 255)
        c_right = (50, 150, 165, 255)
        border = (30, 100, 110, 255)
        draw.polygon([p_top, p_right, p_center, p_left], fill=c_top, outline=border)
        draw.polygon([p_left, p_center, p_b_center, p_b_left], fill=c_left, outline=border)
        draw.polygon([p_center, p_right, p_b_right, p_b_center], fill=c_right, outline=border)
        draw.polygon([(cx, cy - 10), (cx + 16, cy - 4), (cx, cy + 2), (cx - 16, cy - 4)], outline=(180, 255, 255, 255), fill=(130, 250, 250, 255))

    elif theme == "emerald_block":
        c_top = (50, 225, 95, 255)
        c_left = (35, 175, 75, 255)
        c_right = (25, 135, 60, 255)
        border = (15, 80, 35, 255)
        draw.polygon([p_top, p_right, p_center, p_left], fill=c_top, outline=border)
        draw.polygon([p_left, p_center, p_b_center, p_b_left], fill=c_left, outline=border)
        draw.polygon([p_center, p_right, p_b_right, p_b_center], fill=c_right, outline=border)
        draw.polygon([(cx, cy - 10), (cx + 16, cy - 4), (cx, cy + 2), (cx - 16, cy - 4)], outline=(120, 255, 160, 255), fill=(65, 240, 115, 255))

    elif theme == "gold_block":
        c_top = (255, 225, 65, 255)
        c_left = (225, 180, 45, 255)
        c_right = (185, 145, 30, 255)
        border = (120, 95, 20, 255)
        draw.polygon([p_top, p_right, p_center, p_left], fill=c_top, outline=border)
        draw.polygon([p_left, p_center, p_b_center, p_b_left], fill=c_left, outline=border)
        draw.polygon([p_center, p_right, p_b_right, p_b_center], fill=c_right, outline=border)
        draw.polygon([(cx, cy - 10), (cx + 16, cy - 4), (cx, cy + 2), (cx - 16, cy - 4)], outline=(255, 245, 150, 255), fill=(255, 235, 85, 255))

    elif theme == "furnace":
        c_top = (130, 130, 135, 255)
        c_left = (100, 100, 105, 255)
        c_right = (80, 80, 85, 255)
        border = (40, 40, 45, 255)
        draw.polygon([p_top, p_right, p_center, p_left], fill=c_top, outline=border)
        draw.polygon([p_left, p_center, p_b_center, p_b_left], fill=c_left, outline=border)
        draw.polygon([p_center, p_right, p_b_right, p_b_center], fill=c_right, outline=border)
        # Furnace flame opening
        f_tl = (cx - 18, cy + 3)
        f_tr = (cx - 5, cy + 9)
        f_br = (cx - 5, cy + 23)
        f_bl = (cx - 18, cy + 17)
        draw.polygon([f_tl, f_tr, f_br, f_bl], fill=(30, 30, 35, 255), outline=(50, 50, 55, 255))
        draw.polygon([(cx - 14, cy + 11), (cx - 7, cy + 15), (cx - 7, cy + 21), (cx - 14, cy + 17)], fill=(245, 130, 25, 255))
        draw.polygon([(cx - 12, cy + 14), (cx - 9, cy + 16), (cx - 9, cy + 19), (cx - 12, cy + 17)], fill=(255, 220, 60, 255))

    elif theme == "chest":
        c_top = (165, 115, 55, 255)
        c_left = (135, 90, 40, 255)
        c_right = (110, 70, 30, 255)
        border = (50, 35, 15, 255)
        draw.polygon([p_top, p_right, p_center, p_left], fill=c_top, outline=border)
        draw.polygon([p_left, p_center, p_b_center, p_b_left], fill=c_left, outline=border)
        draw.polygon([p_center, p_right, p_b_right, p_b_center], fill=c_right, outline=border)
        draw.rectangle([cx - 4, cy + 6, cx + 4, cy + 16], fill=(210, 210, 215, 255), outline=(70, 70, 75, 255))
        draw.rectangle([cx - 2, cy + 9, cx + 2, cy + 13], fill=(40, 40, 45, 255))

    elif theme == "tnt":
        c_top = (215, 65, 50, 255)
        c_left = (180, 45, 35, 255)
        c_right = (150, 35, 25, 255)
        border = (70, 20, 15, 255)
        draw.polygon([p_top, p_right, p_center, p_left], fill=c_top, outline=border)
        draw.polygon([p_left, p_center, p_b_center, p_b_left], fill=c_left, outline=border)
        draw.polygon([p_center, p_right, p_b_right, p_b_center], fill=c_right, outline=border)
        # White center stripe
        draw.polygon([(cx - w, cy + 5), (cx, cy + h + 5), (cx, cy + h + 15), (cx - w, cy + 15)], fill=(235, 235, 240, 255))
        draw.polygon([(cx, cy + h + 5), (cx + w, cy + 5), (cx + w, cy + 15), (cx, cy + h + 15)], fill=(215, 215, 220, 255))
        # Draw bold TNT letters
        draw.text((cx - 16, cy + 7), "TNT", fill=(20, 20, 20, 255))

    elif theme == "obsidian":
        c_top = (35, 25, 55, 255)
        c_left = (25, 18, 42, 255)
        c_right = (18, 12, 32, 255)
        border = (10, 5, 18, 255)
        draw.polygon([p_top, p_right, p_center, p_left], fill=c_top, outline=border)
        draw.polygon([p_left, p_center, p_b_center, p_b_left], fill=c_left, outline=border)
        draw.polygon([p_center, p_right, p_b_right, p_b_center], fill=c_right, outline=border)
        specks = [(cx - 10, cy - 5), (cx + 8, cy - 2), (cx - 14, cy + 15), (cx + 12, cy + 18), (cx + 4, cy + 8)]
        for sx, sy in specks:
            draw.rectangle([sx, sy, sx + 2, sy + 2], fill=(145, 60, 215, 255))

    elif theme == "omnicraft":
        c_top = (16, 185, 129, 255)     # emerald 500
        c_left = (6, 95, 70, 255)       # emerald 800
        c_right = (4, 47, 46, 255)      # teal 950
        border = (5, 25, 25, 255)
        draw.polygon([p_top, p_right, p_center, p_left], fill=c_top, outline=border)
        draw.polygon([p_left, p_center, p_b_center, p_b_left], fill=c_left, outline=border)
        draw.polygon([p_center, p_right, p_b_right, p_b_center], fill=c_right, outline=border)
        draw.polygon([(cx, cy - 10), (cx + 15, cy - 4), (cx, cy + 2), (cx - 15, cy - 4)], outline=(167, 243, 208, 255), fill=(52, 211, 153, 255))
        draw.ellipse([cx - 17, cy + 6, cx - 7, cy + 18], outline=(52, 211, 153, 255), width=2)

    return img

def gen_diamond_sword():
    _ = (0, 0, 0, 0)
    G = (60, 210, 225, 255)   # Diamond cyan
    GL = (160, 245, 255, 255) # Diamond light
    GD = (30, 150, 175, 255)  # Diamond dark
    W = (130, 85, 45, 255)    # Wood handle
    WL = (165, 115, 65, 255)
    WD = (90, 55, 30, 255)
    B = (35, 35, 40, 255)     # Outline
    
    grid = [
        [_,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  B, GL,  B],
        [_,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  B, GL,  G,  B],
        [_,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B],
        [_,  _,  _,  _,  _,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B,  _],
        [_,  _,  _,  _,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B,  _,  _],
        [_,  _,  _,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B,  _,  _,  _],
        [_,  _,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B,  _,  _,  _,  _],
        [_,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B,  _,  _,  _,  _,  _],
        [_,  _,  _,  B,  B,  B, GL,  G, GD,  B,  _,  _,  _,  _,  _,  _],
        [_,  _,  B,  G,  G,  B, GD, GD,  B,  B,  B,  _,  _,  _,  _,  _],
        [_,  _,  B,  G, WL,  W,  B,  B,  G,  G,  B,  _,  _,  _,  _,  _],
        [_,  _,  _,  B,  W,  W, WD,  B,  G,  B,  _,  _,  _,  _,  _,  _],
        [_,  _,  _,  _,  B, WD,  W,  W,  B,  _,  _,  _,  _,  _,  _,  _],
        [_,  B,  B,  _,  _,  B, WD,  W,  B,  _,  _,  _,  _,  _,  _,  _],
        [B,  G,  G,  B,  _,  _,  B,  B,  _,  _,  _,  _,  _,  _,  _,  _],
        [B,  B,  B,  B,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _],
    ]
    return render_grid_to_image(grid, 64)

def gen_golden_sword():
    _ = (0, 0, 0, 0)
    G = (245, 210, 50, 255)   # Gold
    GL = (255, 245, 140, 255) # Gold light
    GD = (190, 150, 25, 255)  # Gold dark
    W = (130, 85, 45, 255)
    WL = (165, 115, 65, 255)
    WD = (90, 55, 30, 255)
    B = (45, 35, 15, 255)
    
    grid = [
        [_,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  B, GL,  B],
        [_,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  B, GL,  G,  B],
        [_,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B],
        [_,  _,  _,  _,  _,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B,  _],
        [_,  _,  _,  _,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B,  _,  _],
        [_,  _,  _,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B,  _,  _,  _],
        [_,  _,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B,  _,  _,  _,  _],
        [_,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B,  _,  _,  _,  _,  _],
        [_,  _,  _,  B,  B,  B, GL,  G, GD,  B,  _,  _,  _,  _,  _,  _],
        [_,  _,  B,  G,  G,  B, GD, GD,  B,  B,  B,  _,  _,  _,  _,  _],
        [_,  _,  B,  G, WL,  W,  B,  B,  G,  G,  B,  _,  _,  _,  _,  _],
        [_,  _,  _,  B,  W,  W, WD,  B,  G,  B,  _,  _,  _,  _,  _,  _],
        [_,  _,  _,  _,  B, WD,  W,  W,  B,  _,  _,  _,  _,  _,  _,  _],
        [_,  B,  B,  _,  _,  B, WD,  W,  B,  _,  _,  _,  _,  _,  _,  _],
        [B,  G,  G,  B,  _,  _,  B,  B,  _,  _,  _,  _,  _,  _,  _,  _],
        [B,  B,  B,  B,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _],
    ]
    return render_grid_to_image(grid, 64)

def gen_iron_sword():
    _ = (0, 0, 0, 0)
    G = (210, 210, 215, 255)  # Iron
    GL = (245, 245, 250, 255) # Iron light
    GD = (145, 145, 155, 255) # Iron dark
    W = (130, 85, 45, 255)
    WL = (165, 115, 65, 255)
    WD = (90, 55, 30, 255)
    B = (35, 35, 40, 255)
    
    grid = [
        [_,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  B, GL,  B],
        [_,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  B, GL,  G,  B],
        [_,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B],
        [_,  _,  _,  _,  _,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B,  _],
        [_,  _,  _,  _,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B,  _,  _],
        [_,  _,  _,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B,  _,  _,  _],
        [_,  _,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B,  _,  _,  _,  _],
        [_,  _,  _,  _,  _,  _,  B, GL,  G, GD,  B,  _,  _,  _,  _,  _],
        [_,  _,  _,  B,  B,  B, GL,  G, GD,  B,  _,  _,  _,  _,  _,  _],
        [_,  _,  B,  G,  G,  B, GD, GD,  B,  B,  B,  _,  _,  _,  _,  _],
        [_,  _,  B,  G, WL,  W,  B,  B,  G,  G,  B,  _,  _,  _,  _,  _],
        [_,  _,  _,  B,  W,  W, WD,  B,  G,  B,  _,  _,  _,  _,  _,  _],
        [_,  _,  _,  _,  B, WD,  W,  W,  B,  _,  _,  _,  _,  _,  _,  _],
        [_,  B,  B,  _,  _,  B, WD,  W,  B,  _,  _,  _,  _,  _,  _,  _],
        [B,  G,  G,  B,  _,  _,  B,  B,  _,  _,  _,  _,  _,  _,  _,  _],
        [B,  B,  B,  B,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _],
    ]
    return render_grid_to_image(grid, 64)

def gen_netherite_pickaxe():
    _ = (0, 0, 0, 0)
    N = (68, 62, 65, 255)     # Netherite
    NL = (95, 88, 92, 255)
    ND = (45, 40, 42, 255)
    W = (130, 85, 45, 255)
    WL = (165, 115, 65, 255)
    WD = (90, 55, 30, 255)
    B = (25, 22, 24, 255)
    
    grid = [
        [_,  _,  _,  _,  _,  _,  _,  B, NL,  N, ND,  B,  _,  _,  _,  _],
        [_,  _,  _,  _,  _,  B, NL,  N, ND, NL,  N, ND,  B,  _,  _,  _],
        [_,  _,  _,  _,  B, NL,  N,  B,  B,  B,  N, ND, NL,  B,  _,  _],
        [_,  _,  _,  B, NL,  N,  B,  _,  _,  B,  B, NL,  N, ND,  B,  _],
        [_,  _,  B, NL,  N,  B,  _,  _,  _,  _,  B,  B,  N, ND,  B,  _],
        [_,  B, NL,  N,  B,  _,  _,  _,  _,  _,  B, WL,  B,  B,  _,  _],
        [_,  B,  N,  B,  _,  _,  _,  _,  _,  B, WL,  W,  B,  _,  _,  _],
        [B, ND,  B,  _,  _,  _,  _,  _,  B, WL,  W,  B,  _,  _,  _,  _],
        [B,  B,  _,  _,  _,  _,  _,  B, WL,  W,  B,  _,  _,  _,  _,  _],
        [_,  _,  _,  _,  _,  _,  B, WL,  W,  B,  _,  _,  _,  _,  _,  _],
        [_,  _,  _,  _,  _,  B, WL,  W,  B,  _,  _,  _,  _,  _,  _,  _],
        [_,  _,  _,  _,  B, WL,  W,  B,  _,  _,  _,  _,  _,  _,  _,  _],
        [_,  _,  _,  B, WL,  W,  B,  _,  _,  _,  _,  _,  _,  _,  _,  _],
        [_,  _,  B, WL,  W,  B,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _],
        [_,  B, WD,  W,  B,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _],
        [_,  B,  B,  B,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _],
    ]
    return render_grid_to_image(grid, 64)

def gen_golden_apple():
    _ = (0, 0, 0, 0)
    G = (255, 215, 0, 255)
    GL = (255, 250, 160, 255)
    GD = (210, 165, 0, 255)
    S = (120, 80, 30, 255)
    B = (70, 50, 0, 255)
    
    grid = [
        [_, _, _, _, _, _, B, _, _, _, _, _, _, _, _, _],
        [_, _, _, _, _, _, B, S, _, _, _, _, _, _, _, _],
        [_, _, _, _, B, B, S, B, B, B, _, _, _, _, _, _],
        [_, _, _, B, GL, GL, B, GL, GL, GL, B, _, _, _, _, _],
        [_, _, B, GL, GL, G, G, G, GL, GL, G, B, _, _, _, _],
        [_, B, GL, GL, G, G, G, G, G, G, G, G, B, _, _, _],
        [_, B, GL, G, G, G, G, G, G, G, G, GD, B, _, _, _],
        [_, B, G, G, G, G, G, G, G, G, GD, GD, B, _, _, _],
        [_, B, G, G, G, G, G, G, G, GD, GD, GD, B, _, _, _],
        [_, B, G, G, G, G, G, G, GD, GD, GD, GD, B, _, _, _],
        [_, B, GD, G, G, G, G, GD, GD, GD, GD, GD, B, _, _, _],
        [_, _, B, GD, GD, GD, GD, GD, GD, GD, GD, B, _, _, _, _],
        [_, _, _, B, GD, GD, GD, GD, GD, GD, B, _, _, _, _, _],
        [_, _, _, _, B, B, GD, GD, B, B, _, _, _, _, _, _],
        [_, _, _, _, _, _, B, B, _, _, _, _, _, _, _, _],
        [_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _],
    ]
    return render_grid_to_image(grid, 64)

def gen_ender_pearl():
    _ = (0, 0, 0, 0)
    P = (22, 120, 110, 255)
    PL = (90, 215, 195, 255)
    PD = (10, 65, 60, 255)
    C = (175, 255, 240, 255)
    B = (6, 35, 32, 255)
    
    grid = [
        [_, _, _, _, _, B, B, B, B, _, _, _, _, _, _, _],
        [_, _, _, B, B, PL, PL, PL, PL, B, B, _, _, _, _, _],
        [_, _, B, PL, PL, C,  C,  PL, PL, P, B, _, _, _, _, _],
        [_, B, PL, PL, C,  C,  PL, P,  P,  P, P, B, _, _, _, _],
        [_, B, PL, C,  C,  PL, P,  P,  P,  PD, PD, B, _, _, _, _],
        [B, PL, C,  PL, P,  P,  P,  PD, PD, PD, PD, B, _, _, _, _],
        [B, PL, PL, P,  P,  PD, PD, PD, PD, PD, PD, B, _, _, _, _],
        [B, P,  P,  P,  PD, PD, PD, PD, PD, PD, PD, B, _, _, _, _],
        [B, P,  P,  PD, PD, PD, PD, PD, PD, PD, B, _, _, _, _, _],
        [_, B, PD, PD, PD, PD, PD, PD, PD, PD, B, _, _, _, _, _],
        [_, B, PD, PD, PD, PD, PD, PD, PD, B, _, _, _, _, _, _],
        [_, _, B, B, PD, PD, PD, PD, B, B, _, _, _, _, _, _],
        [_, _, _, _, B, B, B, B, _, _, _, _, _, _, _, _],
        [_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _],
        [_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _],
        [_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _],
    ]
    return render_grid_to_image(grid, 64)

def gen_book():
    _ = (0, 0, 0, 0)
    R = (150, 45, 40, 255)
    RL = (195, 65, 60, 255)
    RD = (100, 30, 25, 255)
    P = (235, 230, 215, 255)
    PD = (190, 185, 170, 255)
    G = (245, 215, 65, 255)
    B = (40, 15, 15, 255)
    
    grid = [
        [_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _],
        [_, _, _, _, B, B, B, B, B, B, B, B, _, _, _, _],
        [_, _, _, B, RL, RL, RL, RL, RL, RL, RL, B, _, _, _, _],
        [_, _, B, RL, G,  RL, RL, RL, RL, RL, RL, B, _, _, _],
        [_, B, RL, G,  G,  RL, RL, RL, RL, RL, RL, B, _, _, _],
        [_, B, R,  RL, G,  R,  R,  R,  R,  R,  R,  B, _, _, _],
        [_, B, R,  R,  RL, R,  P,  P,  P,  P,  P,  B, _, _, _],
        [_, B, R,  R,  R,  R,  P,  PD, P,  PD, P,  B, _, _, _],
        [_, B, RD, R,  R,  RD, P,  P,  P,  P,  P,  B, _, _, _],
        [_, B, RD, RD, RD, RD, P,  PD, P,  PD, P,  B, _, _, _],
        [_, B, RD, G,  RD, RD, P,  P,  P,  P,  P,  B, _, _, _],
        [_, _, B, RD, G,  G,  B, B, B, B, B, B, _, _, _, _],
        [_, _, _, B, RD, RD, B, _, _, _, _, _, _, _, _, _],
        [_, _, _, _, B, B, B, _, _, _, _, _, _, _, _, _],
        [_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _],
        [_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _],
    ]
    return render_grid_to_image(grid, 64)

# Create registry of new icons
new_icons = {
    # Key blocks
    "block-crafting.png": gen_isometric_block("crafting_table"),
    "art-Crafting_Table.png": gen_isometric_block("crafting_table"),
    "crafting_table.png": gen_isometric_block("crafting_table"),
    
    "block-grass.png": gen_isometric_block("grass"),
    "block-grass2.png": gen_isometric_block("grass"),
    "art-Grass.png": gen_isometric_block("grass"),
    "grass_block.png": gen_isometric_block("grass"),
    
    "block-Furnace.png": gen_isometric_block("furnace"),
    "furnace.png": gen_isometric_block("furnace"),
    
    "block-chest.png": gen_isometric_block("chest"),
    "chest.png": gen_isometric_block("chest"),
    
    "block-tnt.png": gen_isometric_block("tnt"),
    "art-TNT.png": gen_isometric_block("tnt"),
    "tnt.png": gen_isometric_block("tnt"),
    
    "block-diamond-block.png": gen_isometric_block("diamond_block"),
    "block-Diamond_ore.png": gen_isometric_block("diamond_block"),
    "diamond_block.png": gen_isometric_block("diamond_block"),
    
    "block-emerald-block.png": gen_isometric_block("emerald_block"),
    "emerald_block.png": gen_isometric_block("emerald_block"),
    
    "gold_block.png": gen_isometric_block("gold_block"),
    "obsidian.png": gen_isometric_block("obsidian"),
    
    # Omnicraft signature block
    "omnicraft-logo.png": gen_isometric_block("omnicraft"),
    "omnicraft_cube.png": gen_isometric_block("omnicraft"),
    
    # Avatars / Mobs
    "mini-steve.png": gen_steve(),
    "steve.png": gen_steve(),
    "mini-alex.png": gen_alex(),
    "alex.png": gen_alex(),
    
    "mini-creeper.png": gen_creeper(),
    "art-Creeper.png": gen_creeper(),
    "creeper.png": gen_creeper(),
    
    "mini-skeleton.png": gen_skeleton(),
    "skeleton.png": gen_skeleton(),
    
    "mini-zombie.png": gen_zombie(),
    "art-Zombie.png": gen_zombie(),
    "zombie.png": gen_zombie(),
    
    "mini-enderman.png": gen_enderman(),
    "art-Enderman.png": gen_enderman(),
    "enderman.png": gen_enderman(),
    
    # Items / Weapons
    "art-Diamond_Sword.png": gen_diamond_sword(),
    "diamond_sword.png": gen_diamond_sword(),
    
    "art-Gold_Sword.png": gen_golden_sword(),
    "gold_sword.png": gen_golden_sword(),
    
    "art-Iron_Sword.png": gen_iron_sword(),
    "iron_sword.png": gen_iron_sword(),
    
    "netherite_pickaxe.png": gen_netherite_pickaxe(),
    "golden_apple.png": gen_golden_apple(),
    
    "otro-perla.png": gen_ender_pearl(),
    "ender_pearl.png": gen_ender_pearl(),
    
    "art-Book.png": gen_book(),
    "block-book.png": gen_book(),
    "book.png": gen_book(),
}

print(f"Generated {len(new_icons)} icons. Writing to output directories...")

for out_dir in OUTPUT_DIRS:
    os.makedirs(out_dir, exist_ok=True)
    for filename, img in new_icons.items():
        dest = os.path.join(out_dir, filename)
        img.save(dest, format="PNG")
        
print("Successfully generated and saved all Minecraft icons!")
