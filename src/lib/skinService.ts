// Servicio de Skin y Avatares en el Frontend
export class SkinService {
  static getAvatarUrl(usernameOrUuid: string, size: number = 64): string {
    if (!usernameOrUuid) return '/minecraft-icons/mini-steve.png';
    return `https://mc-heads.net/avatar/${encodeURIComponent(usernameOrUuid)}/${size}`;
  }

  static getBodyUrl(usernameOrUuid: string, height: number = 180): string {
    if (!usernameOrUuid) return '/minecraft-icons/mini-steve.png';
    return `https://mc-heads.net/body/${encodeURIComponent(usernameOrUuid)}/${height}`;
  }
}
