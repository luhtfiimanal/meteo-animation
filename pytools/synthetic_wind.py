#!/usr/bin/env python3
"""
Synthetic Wind Field Visualization with Cartopy

Visualize synthetic wind patterns for POC-6 tile stitching verification.
Generates streamline plots with tile boundary overlays and coastlines.

Usage:
    uv run python synthetic_wind.py [pattern] [zoom]

    pattern: uniform, single, multi (default: multi)
    zoom: 3, 4, 5 (default: 4)

Example:
    uv run python synthetic_wind.py multi 4
"""

import sys
from math import sin, cos, sqrt, atan, atan2, pi, exp, log, tan
import numpy as np
import matplotlib.pyplot as plt
import cartopy.crs as ccrs
import cartopy.feature as cfeature

# =============================================================================
# Constants
# =============================================================================

EARTH_RADIUS = 6378137.0
MERCATOR_EXTENT = 20037508.342789244

# Indonesia region bounds (lon/lat)
LON_MIN, LON_MAX = 80.0, 160.0
LAT_MIN, LAT_MAX = -20.0, 20.0

# meteo-tiler-v3 encoding ranges
CHANNEL_RANGES = {
    'u': {'min': -15, 'max': 15},
    'v': {'min': -15, 'max': 15},
    'mag': {'min': 0, 'max': 35},
}

# Vortex definitions for multi-vortex pattern
VORTICES = [
    # (lon, lat, strength, clockwise)
    (110.0, 0.0, 12.0, True),    # Cyclone di Laut Jawa
    (127.0, -5.0, 10.0, False),  # Anticyclone di Banda
    (135.0, 5.0, 8.0, True),     # Cyclone di Pasifik
]

# Single vortex location
SINGLE_VORTEX_LON = 127.0
SINGLE_VORTEX_LAT = -5.0

# =============================================================================
# Simplified Land Polygons (for NaN regions)
# =============================================================================

LAND_POLYGONS = {
    'sumatra': [
        (95.3, 5.6), (97.5, 5.2), (99.0, 4.0), (100.5, 2.0),
        (102.0, 0.5), (104.0, -1.0), (105.5, -3.0), (106.0, -5.9),
        (104.5, -5.8), (103.0, -4.5), (101.0, -2.5), (99.0, -0.5),
        (97.0, 1.5), (95.0, 3.5), (95.3, 5.6)
    ],
    'java': [
        (105.2, -5.9), (106.5, -6.0), (108.0, -6.2), (110.0, -6.9),
        (112.0, -7.2), (114.0, -7.8), (114.5, -8.2), (113.0, -8.5),
        (110.0, -8.0), (107.5, -7.5), (105.8, -6.8), (105.2, -5.9)
    ],
    'kalimantan': [
        (108.8, 1.0), (110.0, 1.5), (112.0, 1.2), (115.0, 2.5),
        (117.5, 4.2), (118.0, 3.0), (117.5, 1.0), (117.0, -1.0),
        (116.0, -3.0), (114.5, -4.0), (112.0, -3.5), (110.5, -2.5),
        (109.5, -1.0), (108.8, 1.0)
    ],
    'sulawesi': [
        (119.5, 1.5), (120.5, 0.5), (121.0, -1.0), (122.5, -2.0),
        (121.5, -3.5), (120.5, -5.5), (119.5, -5.0), (120.0, -3.0),
        (119.0, -1.5), (119.5, 1.5)
    ],
    'papua': [
        (130.0, -2.5), (132.0, -2.0), (135.0, -3.5), (138.0, -5.0),
        (141.0, -5.5), (141.0, -8.5), (138.0, -8.0), (135.0, -6.0),
        (132.0, -4.5), (130.0, -2.5)
    ],
}


def point_in_polygon(lon, lat, polygon):
    """Ray casting algorithm for point-in-polygon test."""
    n = len(polygon)
    inside = False
    j = n - 1
    for i in range(n):
        xi, yi = polygon[i]
        xj, yj = polygon[j]
        if ((yi > lat) != (yj > lat)) and (lon < (xj - xi) * (lat - yi) / (yj - yi) + xi):
            inside = not inside
        j = i
    return inside


def is_land(lon, lat):
    """Return True if point is on land (simplified polygons)."""
    for polygon in LAND_POLYGONS.values():
        if point_in_polygon(lon, lat, polygon):
            return True
    return False


# =============================================================================
# Coordinate Conversions
# =============================================================================

def lonlat_to_mercator(lon, lat):
    """Convert lon/lat to Web Mercator (EPSG:3857)."""
    x = lon * pi / 180 * EARTH_RADIUS
    lat_rad = lat * pi / 180
    y = log(tan(pi / 4 + lat_rad / 2)) * EARTH_RADIUS
    return x, y


def mercator_to_lonlat(x, y):
    """Convert Web Mercator to lon/lat."""
    lon = x / EARTH_RADIUS * 180 / pi
    lat = (2 * atan(exp(y / EARTH_RADIUS)) - pi / 2) * 180 / pi
    return lon, lat


def tile_to_mercator_bounds(z, x, y):
    """Convert tile coords (z/x/y) to Web Mercator bounds."""
    n = 2 ** z
    world_size = MERCATOR_EXTENT * 2

    tile_size = world_size / n

    min_x = -MERCATOR_EXTENT + x * tile_size
    max_x = min_x + tile_size

    # Y is flipped: y=0 is at top (north)
    max_y = MERCATOR_EXTENT - y * tile_size
    min_y = max_y - tile_size

    return min_x, min_y, max_x, max_y


def get_tiles_in_bounds(z, lon_min, lat_min, lon_max, lat_max):
    """Get all tiles that intersect the given lon/lat bounds at zoom z."""
    # Convert bounds to mercator
    merc_min_x, merc_min_y = lonlat_to_mercator(lon_min, lat_min)
    merc_max_x, merc_max_y = lonlat_to_mercator(lon_max, lat_max)

    n = 2 ** z
    world_size = MERCATOR_EXTENT * 2
    tile_size = world_size / n

    # Calculate tile range
    x_min = int((merc_min_x + MERCATOR_EXTENT) / tile_size)
    x_max = int((merc_max_x + MERCATOR_EXTENT) / tile_size)
    y_min = int((MERCATOR_EXTENT - merc_max_y) / tile_size)
    y_max = int((MERCATOR_EXTENT - merc_min_y) / tile_size)

    tiles = []
    for x in range(max(0, x_min), min(n, x_max + 1)):
        for y in range(max(0, y_min), min(n, y_max + 1)):
            tiles.append((z, x, y))

    return tiles


# =============================================================================
# Wind Patterns
# =============================================================================

def wind_uniform_gradient(lon, lat):
    """Pattern A: Eastward wind with latitudinal variation."""
    u = 5.0  # Base eastward wind (m/s)
    v = 2.0 * sin(lat * pi / 40)  # +/-2 m/s variation
    return u, v


def wind_single_vortex(lon, lat):
    """Pattern B: Single cyclone at Banda Sea."""
    dx = lon - SINGLE_VORTEX_LON
    dy = lat - SINGLE_VORTEX_LAT
    dist = sqrt(dx * dx + dy * dy) + 0.01

    # Tangential velocity (clockwise for Southern Hemisphere cyclone)
    angle = atan2(dy, dx) + pi / 2
    radius = 5.0  # degrees
    strength = 10.0  # m/s
    magnitude = strength * min(1.0, radius / dist) * exp(-dist / radius)

    u = cos(angle) * magnitude
    v = sin(angle) * magnitude
    return u, v


def wind_multi_vortex(lon, lat):
    """Pattern C: Multiple vortices + base trade winds."""
    u, v = 3.0, 0.0  # Base easterly trade wind

    for vx_lon, vx_lat, strength, clockwise in VORTICES:
        dx = lon - vx_lon
        dy = lat - vx_lat
        dist = sqrt(dx * dx + dy * dy) + 0.01

        angle = atan2(dy, dx)
        if clockwise:
            angle += pi / 2
        else:
            angle -= pi / 2

        falloff = min(1.0, 5.0 / dist)
        u += cos(angle) * strength * falloff
        v += sin(angle) * strength * falloff

    return u, v


PATTERNS = {
    'uniform': wind_uniform_gradient,
    'single': wind_single_vortex,
    'multi': wind_multi_vortex,
}


# =============================================================================
# Visualization
# =============================================================================

def create_wind_field(pattern_name, resolution=100, with_land_mask=True):
    """Create u, v wind field arrays for the Indonesia region.

    Returns:
        lons, lats, U, V, validity_mask
        validity_mask: True for ocean (valid), False for land (NaN)
    """
    pattern_func = PATTERNS[pattern_name]

    lons = np.linspace(LON_MIN, LON_MAX, resolution)
    lats = np.linspace(LAT_MIN, LAT_MAX, resolution)

    LON, LAT = np.meshgrid(lons, lats)
    U = np.zeros_like(LON)
    V = np.zeros_like(LAT)
    valid = np.ones_like(LON, dtype=bool)

    for i in range(resolution):
        for j in range(resolution):
            lon, lat = lons[j], lats[i]
            u, v = pattern_func(lon, lat)

            if with_land_mask and is_land(lon, lat):
                U[i, j] = np.nan
                V[i, j] = np.nan
                valid[i, j] = False
            else:
                U[i, j] = u
                V[i, j] = v

    return lons, lats, U, V, valid


def plot_tile_boundaries(ax, zoom, color='red', alpha=0.7, linewidth=1.5):
    """Draw tile boundaries for given zoom level."""
    tiles = get_tiles_in_bounds(zoom, LON_MIN, LAT_MIN, LON_MAX, LAT_MAX)

    for z, x, y in tiles:
        min_x, min_y, max_x, max_y = tile_to_mercator_bounds(z, x, y)

        # Convert to lon/lat
        lon_min, lat_min = mercator_to_lonlat(min_x, min_y)
        lon_max, lat_max = mercator_to_lonlat(max_x, max_y)

        # Clip to visible area
        lon_min = max(lon_min, LON_MIN)
        lon_max = min(lon_max, LON_MAX)
        lat_min = max(lat_min, LAT_MIN)
        lat_max = min(lat_max, LAT_MAX)

        if lon_max > lon_min and lat_max > lat_min:
            # Draw rectangle
            lons = [lon_min, lon_max, lon_max, lon_min, lon_min]
            lats = [lat_min, lat_min, lat_max, lat_max, lat_min]
            ax.plot(lons, lats, color=color, alpha=alpha, linewidth=linewidth,
                    transform=ccrs.PlateCarree())

            # Add tile label
            cx = (lon_min + lon_max) / 2
            cy = (lat_min + lat_max) / 2
            ax.text(cx, cy, f'{z}/{x}/{y}', fontsize=7, ha='center', va='center',
                    color=color, alpha=min(1.0, alpha + 0.2),
                    transform=ccrs.PlateCarree(),
                    bbox=dict(boxstyle='round,pad=0.2', facecolor='white', alpha=0.5))


def plot_nan_regions(ax):
    """Plot simplified land polygons as NaN regions (hatched)."""
    from matplotlib.patches import Polygon as MplPolygon
    from matplotlib.collections import PatchCollection

    patches = []
    for name, coords in LAND_POLYGONS.items():
        poly = MplPolygon(coords, closed=True)
        patches.append(poly)

    collection = PatchCollection(
        patches,
        facecolor='gray',
        edgecolor='darkgray',
        alpha=0.6,
        hatch='///',
        linewidth=1,
        transform=ccrs.PlateCarree(),
    )
    ax.add_collection(collection)


def visualize_pattern(pattern_name, zoom=4, save_path=None, show_nan=True):
    """Create visualization for a wind pattern with Cartopy."""
    print(f"Generating {pattern_name} pattern at zoom {zoom}...")

    # Create wind field (with or without land mask)
    lons, lats, U, V, valid = create_wind_field(pattern_name, resolution=80,
                                                 with_land_mask=show_nan)

    # Calculate speed for coloring (NaN where invalid)
    speed = np.sqrt(U ** 2 + V ** 2)

    # Create figure with Cartopy projection
    fig = plt.figure(figsize=(16, 10))
    ax = fig.add_subplot(1, 1, 1, projection=ccrs.PlateCarree())

    # Set extent
    ax.set_extent([LON_MIN, LON_MAX, LAT_MIN, LAT_MAX], crs=ccrs.PlateCarree())

    # Add cartopy features (base layer)
    ax.add_feature(cfeature.OCEAN, facecolor='lightblue', alpha=0.3)
    ax.add_feature(cfeature.COASTLINE, linewidth=0.8, edgecolor='black')
    ax.add_feature(cfeature.BORDERS, linestyle=':', linewidth=0.5, edgecolor='gray')

    # Plot NaN regions (simplified land polygons) - BEFORE streamlines
    if show_nan:
        plot_nan_regions(ax)

    # Add gridlines
    gl = ax.gridlines(draw_labels=True, linewidth=0.5, color='gray', alpha=0.5)
    gl.top_labels = False
    gl.right_labels = False

    # Plot streamlines (will skip NaN areas automatically)
    strm = ax.streamplot(
        lons, lats, U, V,
        color=speed,
        cmap='plasma',
        linewidth=1.5,
        density=2.5,
        arrowsize=1.2,
        transform=ccrs.PlateCarree(),
    )

    # Add colorbar
    cbar = fig.colorbar(strm.lines, ax=ax, label='Wind Speed (m/s)',
                        orientation='vertical', shrink=0.8, pad=0.02)

    # Plot tile boundaries
    plot_tile_boundaries(ax, zoom, color='red', alpha=0.8, linewidth=2)

    # Mark vortex centers
    if pattern_name == 'single':
        ax.plot(SINGLE_VORTEX_LON, SINGLE_VORTEX_LAT, 'r*', markersize=20,
                transform=ccrs.PlateCarree(), label='Vortex Center',
                markeredgecolor='white', markeredgewidth=1)
        ax.legend(loc='lower right')
    elif pattern_name == 'multi':
        for vx_lon, vx_lat, strength, clockwise in VORTICES:
            marker_color = 'red' if clockwise else 'blue'
            ax.plot(vx_lon, vx_lat, '*', color=marker_color, markersize=18,
                    transform=ccrs.PlateCarree(),
                    markeredgecolor='white', markeredgewidth=1)
        # Manual legend
        ax.plot([], [], 'r*', markersize=15, label='Cyclone (CW)')
        ax.plot([], [], 'b*', markersize=15, label='Anticyclone (CCW)')
        if show_nan:
            from matplotlib.patches import Patch
            ax.legend(handles=[
                plt.Line2D([0], [0], marker='*', color='red', linestyle='None',
                           markersize=15, label='Cyclone (CW)'),
                plt.Line2D([0], [0], marker='*', color='blue', linestyle='None',
                           markersize=15, label='Anticyclone (CCW)'),
                Patch(facecolor='gray', edgecolor='darkgray', hatch='///',
                      alpha=0.6, label='NaN Region (Land)'),
            ], loc='lower right')
        else:
            ax.legend(loc='lower right')

    # Title
    nan_info = ' + NaN regions (hatched)' if show_nan else ''
    ax.set_title(f'Synthetic Wind Field: {pattern_name.upper()}\n'
                 f'Tile boundaries at zoom {zoom} (red rectangles){nan_info}',
                 fontsize=14, fontweight='bold')

    plt.tight_layout()

    if save_path:
        plt.savefig(save_path, dpi=150, bbox_inches='tight')
        print(f"Saved to {save_path}")
    else:
        plt.show()

    plt.close()
    return fig, ax


def main():
    """Main entry point."""
    # Parse arguments
    pattern = 'multi'
    zoom = 4

    if len(sys.argv) > 1:
        pattern = sys.argv[1]
        if pattern not in PATTERNS:
            print(f"Unknown pattern: {pattern}")
            print(f"Available: {', '.join(PATTERNS.keys())}")
            sys.exit(1)

    if len(sys.argv) > 2:
        try:
            zoom = int(sys.argv[2])
        except ValueError:
            print(f"Invalid zoom: {sys.argv[2]}")
            sys.exit(1)

    print("=" * 60)
    print("Synthetic Wind Field Visualization (Cartopy)")
    print("=" * 60)
    print(f"Pattern: {pattern}")
    print(f"Zoom level: {zoom}")
    print(f"Region: Indonesia ({LON_MIN}-{LON_MAX}E, {LAT_MIN}-{LAT_MAX}N)")
    print()

    # Save to file for headless environments
    save_path = f"wind_{pattern}_z{zoom}.png"
    visualize_pattern(pattern, zoom, save_path)

    print("\nDone!")


if __name__ == '__main__':
    main()
