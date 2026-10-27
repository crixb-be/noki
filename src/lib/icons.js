/** Iconos de Noki a elegir en Ajustes → Apariencia (el primero es el oficial). Los PNG de la bandeja están en src-tauri/icons/variants. */
export const APP_ICONS = [
  {
    "id": "oceano",
    "name": "Océano",
    "stops": [
      [
        0,
        "#020025"
      ],
      [
        0.25,
        "#06085F"
      ],
      [
        0.5,
        "#083696"
      ],
      [
        0.75,
        "#0281C6"
      ],
      [
        1,
        "#00CDFA"
      ]
    ]
  },
  {
    "id": "atardecer",
    "name": "Atardecer",
    "stops": [
      [
        0,
        "#F8B042"
      ],
      [
        0.25,
        "#FD6631"
      ],
      [
        0.5,
        "#FD1E1F"
      ],
      [
        0.75,
        "#C02C68"
      ],
      [
        1,
        "#843AB3"
      ]
    ]
  },
  {
    "id": "aurora",
    "name": "Aurora",
    "stops": [
      [
        0,
        "#0DEAC9"
      ],
      [
        0.25,
        "#0193AA"
      ],
      [
        0.5,
        "#073C8A"
      ],
      [
        0.75,
        "#06085F"
      ],
      [
        1,
        "#020025"
      ]
    ]
  },
  {
    "id": "pastel-atardecer",
    "name": "Atardecer pastel",
    "stops": [
      [
        0,
        "#FF8E95"
      ],
      [
        0.5,
        "#FFB784"
      ],
      [
        1,
        "#FFE595"
      ]
    ]
  },
  {
    "id": "pastel-aurora",
    "name": "Aurora pastel",
    "stops": [
      [
        0,
        "#8EA7F3"
      ],
      [
        0.5,
        "#7FC9E6"
      ],
      [
        1,
        "#9DF0CD"
      ]
    ]
  }
];

export const DEFAULT_ICON = "oceano";
export const iconById = (id) => APP_ICONS.find((i) => i.id === id) ?? APP_ICONS[0];
