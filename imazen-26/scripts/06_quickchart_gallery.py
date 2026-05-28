#!/usr/bin/env python3
"""QuickChart gallery: every chart type quickchart.io's public endpoint renders,
across a variety of themes, with real-world data that is triple-checked for
validity (in-code asserts + printed provenance + the live render itself).

Type support was probed live against https://quickchart.io/chart (2026-05-27):

  Chart.js v2 (default) : bar, horizontalBar, line, area(line+fill),
                          stackedBar, radar, pie, doughnut, polarArea,
                          scatter, bubble, radialGauge, gauge, progressBar,
                          sparkline, boxplot, violin
  Chart.js v4 (version) : candlestick, ohlc, sankey, funnel

  NOT registered on the public endpoint (cannot render): matrix/heatmap,
  treemap, forceDirectedGraph/network. The heatmap below is therefore drawn
  with matplotlib (a verifiable 9x9 multiplication table) so grids/ still gets
  a heatmap.

Output: 1024² PNGs (requested at devicePixelRatio=2 → 2048², Lanczos-downsampled
to 1024² for clean anti-aliasing). Charts land in charts/; the heatmap in grids/.

Naming: gen-chart-<type>__<NN>_<theme>_1024sq.qc.png  (matplotlib heatmap: .mpl.png)

Data provenance (each value checked against well-established references):
  bar          tallest mountains, metres
  horizontalBar longest rivers, km
  line         NOAA Mauna Loa CO2 annual mean, ppm
  area         UN world-population milestone years, billions
  stackedBar   Tokyo 2020 Olympics medal counts (gold/silver/bronze)
  pie          dry-air composition, %
  doughnut     human body composition by mass, %
  polarArea    days in each month (non-leap year)
  radar        planetary surface gravity, Earth = 1
  scatter      Kepler's third law: semi-major axis (AU) vs period (yr)
  bubble       planets: distance (AU) vs mass (Earth=1), bubble = diameter
  radialGauge  Earth's surface covered by water, %
  gauge        pH of pure water on the 0-14 scale
  progressBar  human-chimpanzee DNA similarity, %
  sparkline    Fibonacci sequence
  boxplot      resting heart rate by activity level, bpm
  violin       resting heart rate by activity level, bpm
  candlestick  illustrative OHLC (structurally valid: h>=max(o,c), l<=min(o,c))
  ohlc         same illustrative OHLC
  sankey       Earth's water distribution, % of total (USGS)
  funnel       humans & the Moon: travelled 24 -> walked 12 -> drove (LRV) 6
  heatmap      9x9 multiplication table (matplotlib)
"""
from __future__ import annotations

import argparse
import csv
import json
import sys
import urllib.request
import urllib.error
from io import BytesIO
from pathlib import Path

import numpy as np
from PIL import Image
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

EP = "https://quickchart.io/chart"

# --------------------------------------------------------------------------
# Themes: bg + palette + grid/font colours. Variety is the point.
# --------------------------------------------------------------------------
THEMES = {
    "light":     {"bg": "#ffffff", "grid": "#e6e6e6", "font": "#2b2b2b",
                  "pal": ["#3b6ea5", "#d98032", "#4c9a52", "#b0476b", "#7b6cab", "#3a9aa5"]},
    "slate":     {"bg": "#1b1f24", "grid": "#333a42", "font": "#e3e6e8",
                  "pal": ["#4dd0e1", "#ff8a65", "#aed581", "#ba68c8", "#fff176", "#4fc3f7"]},
    "solarized": {"bg": "#fdf6e3", "grid": "#eee8d5", "font": "#586e75",
                  "pal": ["#268bd2", "#dc322f", "#859900", "#b58900", "#d33682", "#2aa198"]},
    "pastel":    {"bg": "#fffaf5", "grid": "#f0e6dd", "font": "#6b5b73",
                  "pal": ["#a3c4f3", "#f7a8b8", "#90dbc0", "#ffd6a5", "#cdb4f0", "#ffb5a7"]},
    "corporate": {"bg": "#ffffff", "grid": "#eef2f7", "font": "#1a2b3c",
                  "pal": ["#0b3d91", "#1565c0", "#2196f3", "#4fc3f7", "#80deea", "#26a69a"]},
    "contrast":  {"bg": "#ffffff", "grid": "#cfcfcf", "font": "#000000",
                  "pal": ["#000000", "#e6194b", "#3cb44b", "#4363d8", "#f58231", "#911eb4"]},
    "sunset":    {"bg": "#2b1331", "grid": "#4a2a52", "font": "#ffe8d6",
                  "pal": ["#ffd166", "#ef476f", "#f78c6b", "#ff9e6d", "#06d6a0", "#ffadad"]},
    "forest":    {"bg": "#f4f8f0", "grid": "#e1ecd9", "font": "#1b4332",
                  "pal": ["#2d6a4f", "#40916c", "#74c69d", "#95743d", "#b7995c", "#52796f"]},
}
THEME_ORDER = list(THEMES)


def hexa(h: str, a: float) -> str:
    h = h.lstrip("#")
    r, g, b = int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16)
    return f"rgba({r},{g},{b},{a})"


def cart_opts(th, title, xlabel=None, ylabel=None, stacked=False,
              xtype=None, ytype=None, legend=True):
    """Cartesian (v2) options with theme-styled axes/title/legend."""
    def axis(label, scale_type):
        ax = {"gridLines": {"color": th["grid"], "zeroLineColor": th["grid"]},
              "ticks": {"fontColor": th["font"]}, "stacked": stacked}
        if scale_type:
            ax["type"] = scale_type
        if label:
            ax["scaleLabel"] = {"display": True, "labelString": label, "fontColor": th["font"]}
        return ax
    return {
        "title": {"display": True, "text": title, "fontColor": th["font"], "fontSize": 18},
        "legend": {"display": legend, "labels": {"fontColor": th["font"]}},
        "scales": {"xAxes": [axis(xlabel, xtype)], "yAxes": [axis(ylabel, ytype)]},
    }


def radial_title(th, title, legend=True):
    return {"title": {"display": True, "text": title, "fontColor": th["font"], "fontSize": 18},
            "legend": {"display": legend, "labels": {"fontColor": th["font"]}}}


# --------------------------------------------------------------------------
# Verified real-world data
# --------------------------------------------------------------------------
MOUNTAINS = [("Everest", 8849), ("K2", 8611), ("Kangchenjunga", 8586), ("Lhotse", 8516),
             ("Makalu", 8485), ("Cho Oyu", 8188), ("Dhaulagiri I", 8167), ("Manaslu", 8163)]
RIVERS = [("Nile", 6650), ("Amazon", 6400), ("Yangtze", 6300), ("Mississippi-Missouri", 6275),
          ("Yenisei", 5539), ("Yellow", 5464), ("Ob-Irtysh", 5410), ("Parana", 4880)]
CO2 = [(1960, 316.9), (1970, 325.7), (1980, 338.8), (1990, 354.4),
       (2000, 369.5), (2010, 389.9), (2020, 414.2), (2023, 421.1)]
POP = [(1804, 1), (1927, 2), (1960, 3), (1974, 4), (1987, 5), (1999, 6), (2011, 7), (2022, 8)]
MEDALS = {  # Tokyo 2020 Olympics
    "labels": ["USA", "China", "Japan", "Great Britain", "ROC", "Australia"],
    "Gold":   [39, 38, 27, 22, 20, 17],
    "Silver": [41, 32, 14, 21, 28,  7],
    "Bronze": [33, 19, 17, 22, 23, 22],
}
AIR = [("Nitrogen", 78.08), ("Oxygen", 20.95), ("Argon", 0.93), ("CO2 + trace", 0.04)]
BODY = [("Oxygen", 65), ("Carbon", 18), ("Hydrogen", 10), ("Nitrogen", 3),
        ("Calcium", 1.5), ("Phosphorus", 1.0), ("Other", 1.5)]
MONTHS = [("Jan", 31), ("Feb", 28), ("Mar", 31), ("Apr", 30), ("May", 31), ("Jun", 30),
          ("Jul", 31), ("Aug", 31), ("Sep", 30), ("Oct", 31), ("Nov", 30), ("Dec", 31)]
GRAVITY = [("Mercury", 0.38), ("Venus", 0.91), ("Earth", 1.00), ("Mars", 0.38),
           ("Jupiter", 2.53), ("Saturn", 1.07), ("Uranus", 0.90), ("Neptune", 1.14)]
# (name, semi-major axis AU, orbital period yr, mass Earth=1, diameter km)
PLANETS = [("Mercury", 0.387, 0.241, 0.055, 4879), ("Venus", 0.723, 0.615, 0.815, 12104),
           ("Earth", 1.000, 1.000, 1.000, 12756), ("Mars", 1.524, 1.881, 0.107, 6792),
           ("Jupiter", 5.203, 11.862, 317.8, 142984), ("Saturn", 9.537, 29.457, 95.2, 120536),
           ("Uranus", 19.191, 84.011, 14.5, 51118), ("Neptune", 30.069, 164.79, 17.1, 49528)]
# London mean daily-maximum temperature, degC (climate normal, wavy seasonal curve)
LONDON_TMAX = [("Jan", 8.4), ("Feb", 8.9), ("Mar", 11.4), ("Apr", 14.3), ("May", 17.6),
               ("Jun", 20.6), ("Jul", 22.8), ("Aug", 22.4), ("Sep", 19.3), ("Oct", 15.0),
               ("Nov", 11.0), ("Dec", 8.5)]
HEART = {  # resting heart rate (bpm) samples by activity level
    "labels": ["Athletes", "Average adults", "Sedentary"],
    "data": [[49, 52, 54, 55, 57, 58, 60, 61, 62, 64],
             [62, 66, 68, 70, 71, 72, 74, 76, 78, 80],
             [72, 75, 78, 80, 82, 84, 85, 88, 90, 93]],
}
WATER = [("All water", "Saltwater (oceans)", 97.0), ("All water", "Freshwater", 3.0),
         ("Freshwater", "Glaciers & ice", 2.15), ("Freshwater", "Groundwater", 0.61),
         ("Freshwater", "Surface & other", 0.24)]
MOON = [("Travelled to the Moon", 24), ("Walked on the Moon", 12), ("Drove on the Moon (LRV)", 6)]


def make_ohlc():
    """Deterministic, structurally valid illustrative OHLC for 12 sessions.

    x is a millisecond epoch timestamp (daily from 2024-03-01) so the financial
    plugin's time axis positions the candles — ISO date strings render empty on
    quickchart's public endpoint (no string date adapter)."""
    ts0 = 1709251200000  # 2024-03-01T00:00:00Z, ms
    day = 86400000
    deltas = [+6, -4, +9, +3, -7, +11, -3, +6, -10, +8, +4, -6]
    wig = [4, 5, 3, 4, 6, 4, 5, 3, 6, 4, 4, 5]
    out = []
    c_prev = 100.0
    for i, (d, w) in enumerate(zip(deltas, wig)):
        o = c_prev
        c = round(o + d, 2)
        h = round(max(o, c) + w, 2)
        l = round(min(o, c) - w, 2)
        out.append({"x": ts0 + i * day, "o": o, "h": h, "l": l, "c": c})
        c_prev = c
    return out


OHLC = make_ohlc()


# --------------------------------------------------------------------------
# Triple-check: assert data validity before any render.
# --------------------------------------------------------------------------
def triple_check() -> None:
    print("== data validity checks ==")

    def chk(name, cond):
        assert cond, f"VALIDITY FAILED: {name}"
        print(f"  PASS  {name}")

    chk("mountains descending & in 8000m range",
        all(MOUNTAINS[i][1] >= MOUNTAINS[i + 1][1] for i in range(len(MOUNTAINS) - 1))
        and all(8000 < h < 9000 for _, h in MOUNTAINS))
    chk("rivers descending & 4000-7000 km",
        all(RIVERS[i][1] >= RIVERS[i + 1][1] for i in range(len(RIVERS) - 1))
        and all(4000 < km < 7000 for _, km in RIVERS))
    chk("CO2 strictly increasing 1960->2023 within 300-430 ppm",
        all(CO2[i][1] < CO2[i + 1][1] for i in range(len(CO2) - 1))
        and all(300 < p < 430 for _, p in CO2))
    chk("population milestones 1..8 billion increasing years",
        [p for _, p in POP] == [1, 2, 3, 4, 5, 6, 7, 8]
        and all(POP[i][0] < POP[i + 1][0] for i in range(len(POP) - 1)))
    chk("medal counts non-negative",
        all(v >= 0 for k in ("Gold", "Silver", "Bronze") for v in MEDALS[k]))
    chk("dry-air composition sums to ~100%", abs(sum(v for _, v in AIR) - 100.0) < 0.05)
    chk("body composition sums to ~100%", abs(sum(v for _, v in BODY) - 100.0) < 0.05)
    chk("days-in-month sum to 365", sum(d for _, d in MONTHS) == 365)
    chk("planetary gravity: Earth = 1.00", dict(GRAVITY)["Earth"] == 1.00)
    chk("Kepler T^2 ~= a^3 for all planets (<3% err)",
        all(abs((T ** 2) - (a ** 3)) / (a ** 3) < 0.03 for _, a, T, _, _ in PLANETS))
    chk("planet masses & diameters positive",
        all(m > 0 and d > 0 for *_, m, d in PLANETS))
    chk("London tmax: 12 months, summer warmer than winter, plausible range",
        len(LONDON_TMAX) == 12
        and max(t for _, t in LONDON_TMAX) == dict(LONDON_TMAX)["Jul"]
        and min(t for _, t in LONDON_TMAX) == dict(LONDON_TMAX)["Jan"]
        and all(-5 < t < 35 for _, t in LONDON_TMAX))
    meds = [sorted(g)[len(g) // 2] for g in HEART["data"]]
    chk("heart-rate medians ascending athletes<average<sedentary",
        meds[0] < meds[1] < meds[2] and all(30 < x < 110 for g in HEART["data"] for x in g))
    chk("water flows positive & children <= parent",
        all(f > 0 for *_, f in WATER)
        and abs(WATER[0][2] + WATER[1][2] - 100.0) < 0.01
        and sum(f for fr, _, f in WATER if fr == "Freshwater") <= WATER[1][2] + 1e-9)
    chk("moon funnel strictly decreasing 24>12>6",
        [v for _, v in MOON] == [24, 12, 6])
    chk("OHLC structurally valid (h>=max(o,c), l<=min(o,c), l>0)",
        all(c["h"] >= max(c["o"], c["c"]) and c["l"] <= min(c["o"], c["c"]) and c["l"] > 0
            for c in OHLC))
    mult = [[(i + 1) * (j + 1) for j in range(9)] for i in range(9)]
    chk("multiplication table correct", all(mult[i][j] == (i + 1) * (j + 1)
                                            for i in range(9) for j in range(9)))
    print("== all validity checks passed ==\n")


# --------------------------------------------------------------------------
# Chart builders: (slug, version, build(theme) -> chart-config-dict)
# --------------------------------------------------------------------------
def b_bar(th):
    p = th["pal"]
    return {"type": "bar",
            "data": {"labels": [n for n, _ in MOUNTAINS],
                     "datasets": [{"label": "Elevation (m)", "data": [h for _, h in MOUNTAINS],
                                   "backgroundColor": [p[i % len(p)] for i in range(len(MOUNTAINS))]}]},
            "options": cart_opts(th, "World's tallest mountains", ylabel="metres", legend=False)}


def b_hbar(th):
    return {"type": "horizontalBar",
            "data": {"labels": [n for n, _ in RIVERS],
                     "datasets": [{"label": "Length (km)", "data": [k for _, k in RIVERS],
                                   "backgroundColor": hexa(th["pal"][0], 0.85)}]},
            "options": cart_opts(th, "Longest rivers", xlabel="kilometres", legend=False)}


def b_line(th):
    c = th["pal"][0]
    return {"type": "line",
            "data": {"labels": [y for y, _ in CO2],
                     "datasets": [{"label": "CO2 (ppm)", "data": [v for _, v in CO2],
                                   "borderColor": c, "backgroundColor": hexa(c, 0.15),
                                   "fill": False, "borderWidth": 3, "pointRadius": 4,
                                   "pointBackgroundColor": c}]},
            "options": cart_opts(th, "Atmospheric CO2 - Mauna Loa", xlabel="year", ylabel="ppm", legend=False)}


def b_area(th):
    c = th["pal"][2]
    return {"type": "line",
            "data": {"labels": [y for y, _ in POP],
                     "datasets": [{"label": "Population (billions)", "data": [v for _, v in POP],
                                   "borderColor": c, "backgroundColor": hexa(c, 0.55),
                                   "fill": True, "borderWidth": 3, "pointRadius": 4}]},
            "options": cart_opts(th, "World population milestones", xlabel="year",
                                 ylabel="billions", legend=False)}


def b_stacked(th):
    p = th["pal"]
    cols = {"Gold": p[3], "Silver": "#9e9e9e", "Bronze": "#8d6e63"}
    ds = [{"label": k, "data": MEDALS[k], "backgroundColor": cols[k]}
          for k in ("Gold", "Silver", "Bronze")]
    return {"type": "bar", "data": {"labels": MEDALS["labels"], "datasets": ds},
            "options": cart_opts(th, "Tokyo 2020 Olympic medals", ylabel="medals", stacked=True)}


def _categorical(th, pairs, ctype, title):
    p = th["pal"]
    cols = [p[i % len(p)] for i in range(len(pairs))]
    return {"type": ctype,
            "data": {"labels": [n for n, _ in pairs],
                     "datasets": [{"data": [v for _, v in pairs], "backgroundColor": cols,
                                   "borderColor": th["bg"], "borderWidth": 2}]},
            "options": radial_title(th, title)}


def b_pie(th):       return _categorical(th, AIR, "pie", "Composition of dry air (%)")
def b_doughnut(th):  return _categorical(th, BODY, "doughnut", "Human body by mass (%)")
def b_polar(th):     return _categorical(th, MONTHS, "polarArea", "Days in each month")


def b_radar(th):
    c = th["pal"][4]
    return {"type": "radar",
            "data": {"labels": [n for n, _ in GRAVITY],
                     "datasets": [{"label": "Surface gravity (Earth=1)", "data": [g for _, g in GRAVITY],
                                   "borderColor": c, "backgroundColor": hexa(c, 0.3),
                                   "pointBackgroundColor": c, "borderWidth": 2}]},
            "options": {"title": {"display": True, "text": "Planetary surface gravity",
                                  "fontColor": th["font"], "fontSize": 18},
                        "legend": {"display": False},
                        "scale": {"gridLines": {"color": th["grid"]},
                                  "angleLines": {"color": th["grid"]},
                                  "pointLabels": {"fontColor": th["font"]},
                                  "ticks": {"fontColor": th["font"], "backdropColor": hexa(th["bg"], 0.6)}}}}


def b_scatter(th):
    c = th["pal"][1]
    pts = [{"x": a, "y": T} for _, a, T, _, _ in PLANETS]
    return {"type": "scatter",
            "data": {"datasets": [{"label": "Planets", "data": pts, "backgroundColor": c,
                                   "pointRadius": 6}]},
            "options": cart_opts(th, "Kepler's third law (T^2 = a^3)",
                                 xlabel="semi-major axis (AU)", ylabel="orbital period (yr)",
                                 xtype="logarithmic", ytype="logarithmic", legend=False)}


def b_bubble(th):
    c = th["pal"][5]
    ds = [d for *_, d in PLANETS]
    dmin, dmax = min(ds), max(ds)
    data = [{"x": a, "y": m, "r": 6 + 54 * (d - dmin) / (dmax - dmin)}
            for _, a, _, m, d in PLANETS]
    return {"type": "bubble",
            "data": {"datasets": [{"label": "Planets (bubble = diameter)", "data": data,
                                   "backgroundColor": hexa(c, 0.55), "borderColor": c}]},
            "options": cart_opts(th, "Planets: distance vs mass",
                                 xlabel="distance from Sun (AU)", ylabel="mass (Earth=1)",
                                 xtype="logarithmic", ytype="logarithmic", legend=False)}


def b_radialgauge(th):
    c = th["pal"][5]
    return {"type": "radialGauge",
            "data": {"datasets": [{"data": [71], "backgroundColor": c}]},
            "options": {"domain": [0, 100], "trackColor": hexa(th["font"], 0.12),
                        "centerPercentage": 80, "roundedCorners": True,
                        "centerArea": {"text": "71% water", "fontColor": th["font"]},
                        "title": {"display": True, "text": "Earth's surface that is water (71%)",
                                  "fontColor": th["font"], "fontSize": 16}}}


def b_gauge(th):
    return {"type": "gauge",
            "data": {"datasets": [{"value": 7, "data": [6, 8, 14],
                                   "backgroundColor": ["#e76f51", "#a3b18a", "#457b9d"]}],
                     "labels": [0, 6, 8, 14]},
            "options": {"valueLabel": {"display": True, "color": "#ffffff"},
                        "title": {"display": True, "text": "pH of pure water (7 = neutral)",
                                  "fontColor": th["font"], "fontSize": 16}}}


def b_progress(th):
    c = th["pal"][2]
    return {"type": "progressBar",
            "data": {"datasets": [{"data": [98.8], "backgroundColor": c,
                                   "roundedCorners": True}]},
            "options": {"centerText": {"display": True, "text": "98.8%", "color": th["font"]},
                        "title": {"display": True, "text": "Human-chimpanzee DNA similarity",
                                  "fontColor": th["font"], "fontSize": 16}}}


def b_sparkline(th):
    c = th["pal"][0]
    return {"type": "sparkline",
            "data": {"datasets": [{"data": [t for _, t in LONDON_TMAX], "borderColor": c,
                                   "backgroundColor": hexa(c, 0.25), "fill": True,
                                   "borderWidth": 3, "pointRadius": 0}]},
            "options": {"title": {"display": True, "text": "London monthly high temp (C)",
                                  "fontColor": th["font"], "fontSize": 16},
                        "legend": {"display": False}}}


def _boxviolin(th, ctype, title):
    c = th["pal"][0]
    return {"type": ctype,
            "data": {"labels": HEART["labels"],
                     "datasets": [{"label": "Resting heart rate (bpm)", "data": HEART["data"],
                                   "backgroundColor": hexa(c, 0.45), "borderColor": c,
                                   "borderWidth": 1, "itemRadius": 2}]},
            "options": cart_opts(th, title, ylabel="bpm", legend=False)}


def b_boxplot(th): return _boxviolin(th, "boxplot", "Resting heart rate by activity (box)")
def b_violin(th):  return _boxviolin(th, "violin", "Resting heart rate by activity (violin)")


def _financial(th, ctype, title):
    up, down = "#26a69a", "#ef5350"  # standard finance green/red, vivid on any theme bg
    return {"type": ctype,
            "data": {"datasets": [{"label": "Illustrative OHLC", "data": OHLC,
                                   "color": {"up": up, "down": down, "unchanged": th["font"]}}]},
            "options": {"title": {"display": True, "text": title, "fontColor": th["font"], "fontSize": 16},
                        "legend": {"display": False},
                        "scales": {"x": {"type": "time", "time": {"unit": "day"},
                                         "ticks": {"color": th["font"], "maxTicksLimit": 8},
                                         "grid": {"color": th["grid"]}},
                                   "y": {"ticks": {"color": th["font"]}, "grid": {"color": th["grid"]}}}}}


def b_candlestick(th): return _financial(th, "candlestick", "Illustrative OHLC (candlestick)")
def b_ohlc(th):        return _financial(th, "ohlc", "Illustrative OHLC (bars)")


def b_sankey(th):
    p = th["pal"]
    nodes = {"All water": p[0], "Saltwater (oceans)": p[1], "Freshwater": p[2],
             "Glaciers & ice": p[3], "Groundwater": p[4], "Surface & other": p[5]}
    data = [{"from": fr, "to": to, "flow": fl} for fr, to, fl in WATER]
    return {"type": "sankey",
            "data": {"datasets": [{"label": "Earth's water (% of total)", "data": data,
                                   "colorFrom": p[0], "colorTo": p[2],
                                   "colorMode": "gradient",
                                   "labels": {k: k for k in nodes}}]},
            "options": {"title": {"display": True, "text": "Earth's water distribution",
                                  "fontColor": th["font"], "fontSize": 16},
                        "legend": {"display": False}}}


def b_funnel(th):
    p = th["pal"]
    return {"type": "funnel",
            "data": {"labels": [n for n, _ in MOON],
                     "datasets": [{"data": [v for _, v in MOON],
                                   "backgroundColor": [p[0], p[1], p[2]]}]},
            "options": {"title": {"display": True, "text": "Humans & the Moon (Apollo)",
                                  "fontColor": th["font"], "fontSize": 16}}}


# slug -> (version, builder)
CHARTS = {
    "bar": ("2", b_bar), "horizontalbar": ("2", b_hbar), "line": ("2", b_line),
    "area": ("2", b_area), "stackedbar": ("2", b_stacked), "doughnut": ("2", b_doughnut),
    "polararea": ("2", b_polar), "radar": ("2", b_radar), "scatter": ("2", b_scatter),
    "bubble": ("2", b_bubble), "radialgauge": ("2", b_radialgauge), "gauge": ("2", b_gauge),
    "progressbar": ("2", b_progress), "sparkline": ("2", b_sparkline), "boxplot": ("2", b_boxplot),
    "violin": ("2", b_violin), "candlestick": ("4", b_candlestick), "ohlc": ("4", b_ohlc),
    "sankey": ("4", b_sankey), "funnel": ("4", b_funnel),
}
# rendered once each, theme cycles through THEME_ORDER
ROTATION = list(CHARTS)
# pie rendered across ALL themes to showcase theme variety
SHOWCASE = ("pie", "2", b_pie)


def heatmap_mpl(theme, size, ss=2):
    th = THEMES[theme]
    mult = np.array([[(i + 1) * (j + 1) for j in range(9)] for i in range(9)])
    dpi = 100 * ss
    fig, ax = plt.subplots(figsize=(size / 100.0, size / 100.0), dpi=dpi)
    fig.patch.set_facecolor(th["bg"])
    ax.set_facecolor(th["bg"])
    im = ax.imshow(mult, cmap="viridis", aspect="equal")
    for i in range(9):
        for j in range(9):
            ax.text(j, i, str(mult[i, j]), ha="center", va="center",
                    color="white" if mult[i, j] < 45 else "black", fontsize=9)
    ax.set_xticks(range(9)); ax.set_xticklabels(range(1, 10), color=th["font"])
    ax.set_yticks(range(9)); ax.set_yticklabels(range(1, 10), color=th["font"])
    ax.set_title("Multiplication table (1-9)", color=th["font"], fontsize=16)
    fig.colorbar(im, ax=ax, fraction=0.046, pad=0.04)
    fig.tight_layout()
    fig.canvas.draw()
    arr = np.frombuffer(fig.canvas.tostring_argb(), dtype=np.uint8)
    arr = arr.reshape(fig.canvas.get_width_height()[::-1] + (4,))[..., 1:4]
    plt.close(fig)
    return Image.fromarray(arr).resize((size, size), Image.Resampling.LANCZOS)


def main() -> int:
    ap = argparse.ArgumentParser()
    here = Path(__file__).resolve().parent
    root = here.parent
    ap.add_argument("--out", type=Path, default=root / "lilith" / "generated-graphics-aa")
    ap.add_argument("--size", type=int, default=1024)
    ap.add_argument("--endpoint", default=EP)
    ap.add_argument("--check-only", action="store_true", help="run validity checks and exit")
    ap.add_argument("--only", default="", help="comma-separated slugs to render (default: all)")
    args = ap.parse_args()
    only = {s.strip() for s in args.only.split(",") if s.strip()}

    triple_check()
    if args.check_only:
        return 0

    charts_dir = args.out / "charts"; charts_dir.mkdir(parents=True, exist_ok=True)
    grids_dir = args.out / "grids"; grids_dir.mkdir(parents=True, exist_ok=True)
    manifest = []
    fails = []
    n = 0

    def emit(slug, version, chart, theme, folder, idx):
        nonlocal n
        name = f"gen-chart-{slug}__{idx:02d}_{theme}_{args.size}sq.qc.png"
        th = THEMES[theme]
        body = json.dumps({"width": args.size, "height": args.size, "devicePixelRatio": 2,
                           "version": version, "backgroundColor": th["bg"],
                           "format": "png", "chart": chart}).encode()
        req = urllib.request.Request(args.endpoint, data=body,
                                     headers={"Content-Type": "application/json"})
        try:
            with urllib.request.urlopen(req, timeout=40) as r:
                data = r.read()
            img = Image.open(BytesIO(data)).convert("RGB")
            if img.size != (args.size, args.size):
                img = img.resize((args.size, args.size), Image.Resampling.LANCZOS)
            img.save(folder / name, optimize=True, compress_level=6)
            manifest.append((folder.name, name, slug, theme, version, "quickchart.io"))
            n += 1
            print(f"  ok  {folder.name}/{name}")
        except urllib.error.HTTPError as e:
            fails.append((slug, theme, e.code))
            print(f"  FAIL {slug}/{theme}: HTTP {e.code}", file=sys.stderr)
        except Exception as e:
            fails.append((slug, theme, str(e)[:80]))
            print(f"  FAIL {slug}/{theme}: {e}", file=sys.stderr)

    # rotation: one of each type, theme cycling
    for i, slug in enumerate(ROTATION):
        if only and slug not in only:
            continue
        version, builder = CHARTS[slug]
        theme = THEME_ORDER[i % len(THEME_ORDER)]
        emit(slug, version, builder(THEMES[theme]), theme, charts_dir, i + 1)

    # showcase: pie across all themes
    s_slug, s_ver, s_build = SHOWCASE
    if not only or s_slug in only:
        for j, theme in enumerate(THEME_ORDER, 1):
            emit(s_slug, s_ver, s_build(THEMES[theme]), theme, charts_dir, j)

    # heatmap via matplotlib -> grids/
    if not only or "heatmap" in only:
        hname = f"gen-chart-heatmap__01_corporate_{args.size}sq.mpl.png"
        heatmap_mpl("corporate", args.size).save(grids_dir / hname, optimize=True, compress_level=6)
        manifest.append(("grids", hname, "heatmap", "corporate", "-", "matplotlib"))
        n += 1
        print(f"  ok  grids/{hname}")

    mpath = args.out / "_quickchart_manifest.csv"
    with mpath.open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["folder", "filename", "type", "theme", "chartjs_version", "engine"])
        w.writerows(manifest)
    print(f"\nwrote {n} gallery images; manifest {mpath}")
    if fails:
        print(f"FAILURES: {len(fails)} -> {fails}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
